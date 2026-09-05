"""Extend the existing friction flow with time-varying physical conditions.

Published nominal sensor clock, not microsecond acoustic synchronization.
Offline research; fixed rubber probe, three known surfaces. No source audio is
required by render. Powered by Stability AI; TangoFlux / Hung et al.; Eguchi et al.
"""

from __future__ import annotations

import argparse
import json
import shutil
import time
from pathlib import Path

import numpy as np
import physical_sound_texture_flow as flow
import soundfile as sf
import torch
from safetensors.torch import save_file
from scipy.signal import resample_poly

FORMAT = "texture-timed-flow-v1"
RATE, HOP = flow.codec.RATE, flow.HOP
PLAYBACK = 50 / flow.GAIN  # One common gain, including the previously omitted edges.


def trace_features(texture, speed, force):
    speed, force = np.asarray(speed), np.asarray(force)
    if (
        texture not in flow.spectrum.TEXTURES
        or speed.ndim != 1
        or speed.shape != force.shape
        or not 32 <= len(speed) <= 256
    ):
        raise ValueError("bounded single-surface physical sequence required")
    if (
        not np.isfinite(speed).all()
        or not np.isfinite(force).all()
        or (speed < 0).any()
        or (speed > 80).any()
        or (force < 0).any()
        or (force > 1.5).any()
    ):
        raise ValueError("speed/force outside observed-transition domain")
    material = np.repeat(
        np.array([texture == x for x in flow.spectrum.TEXTURES])[:, None],
        len(speed),
        axis=1,
    )
    return np.concatenate(
        (material, ((speed - 40) / 20)[None], ((force - 0.75) / 0.25)[None])
    ).astype(np.float32)


def position_speed(position, times):
    """100ms average displacement speed; not a calibrated instantaneous velocity."""
    return (
        np.abs(
            np.interp(times + 0.05, position["time"], position["Y"])
            - np.interp(times - 0.05, position["time"], position["Y"])
        )
        / 0.1
    )


def profile(texture=74, speed=40.0, force=0.5, start=0.3, distance=90.0, tail=0.5):
    flow.spectrum.features(texture, speed, force)
    if (
        not np.isfinite([start, distance, tail]).all()
        or not 0.2 <= start <= 1
        or not 60 <= distance <= 100
        or not 0.2 <= tail <= 1
    ):
        raise ValueError("outside bounded single-slide request")
    ramp = 0.1
    stop = start + distance / speed + ramp
    duration = stop + tail
    count = int(np.ceil(duration * RATE / HOP))
    # Integrate the requested trapezoid on a fixed1ms grid, then use the same
    # 100ms average as the source features. No source trace is loaded.
    tt = np.arange(0, duration + 0.101, 0.001)
    vv = speed * np.minimum(
        np.clip((tt - start) / ramp, 0, 1), np.clip((stop - tt) / ramp, 0, 1)
    )
    yy = 45 - np.concatenate(([0.0], np.cumsum((vv[1:] + vv[:-1]) * 0.0005)))
    position = np.zeros(len(tt), dtype=[("time", float), ("Y", float)])
    position["time"], position["Y"] = tt, yy
    times = (np.arange(count) + 0.5) * HOP / RATE
    physical = trace_features(
        texture, position_speed(position, times), np.full(count, force)
    )
    return physical, {
        "texture": texture,
        "speed_mm_s": speed,
        "force_N": force,
        "start_seconds": start,
        "stop_seconds": stop,
        "duration_seconds": duration,
        "distance_mm": distance,
        "ramp_seconds": ramp,
    }


def envelope(wave):
    mono = resample_poly(wave.mean(1), 1, 2)
    hop = 441  # 20ms in the shared22.05kHz band.
    chunks = mono[: len(mono) // hop * hop].reshape(-1, hop)
    return 10 * np.log10(np.maximum(np.mean(chunks**2, axis=1), 1e-18))


def event_metrics(candidate, reference, physical, commanded_speed):
    actual, real = envelope(candidate), envelope(reference)
    if actual.shape != real.shape:
        raise ValueError("event duration mismatch")
    times = (np.arange(len(real)) + 0.5) * 0.02
    v = np.interp(
        times, (np.arange(physical.shape[1]) + 0.5) * HOP / RATE, physical[3] * 20 + 40
    )
    moving = v > commanded_speed * 0.1
    indices = np.flatnonzero(moving)
    if len(indices) < 3:
        raise ValueError("missing slide")
    before, after = times < times[indices[0]], times > times[indices[-1]]
    out = {
        "envelope_db_mae": float(np.mean(np.abs(actual - real))),
        "envelope_correlation": None
        if min(np.std(actual), np.std(real)) < 1e-12
        else float(np.corrcoef(actual, real)[0, 1]),
        "sensor_motion_start_seconds": float(times[indices[0]]),
        "sensor_motion_stop_seconds": float(times[indices[-1]]),
        "pre_bins": int(before.sum()),
        "post_bins": int(after.sum()),
    }
    for name, mask in (("pre", before), ("post", after)):
        out[name + "_contrast_error_db"] = (
            None
            if not mask.any()
            else float(
                abs(
                    (np.median(actual[moving]) - np.median(actual[mask]))
                    - (np.median(real[moving]) - np.median(real[mask]))
                )
            )
        )
    # Source-defined half-rise diagnostic. Weak contrasts/censored offsets are
    # reported as unavailable, not silently accepted or given fitted thresholds.
    contrast = (
        float(np.median(real[moving]) - np.median(real[before])) if before.any() else 0
    )
    out["reference_rise_db"] = contrast

    def crossings(env, threshold):
        above = env > threshold
        sustained = np.flatnonzero(
            np.convolve(above.astype(int), np.ones(3, int), mode="valid") == 3
        )
        return (
            (None, None)
            if not len(sustained)
            else (float(times[sustained[0]]), float(times[sustained[-1] + 2]))
        )

    if contrast >= 3:
        threshold = float(np.median(real[before]) + contrast / 2)
        rstart, rstop = crossings(real, threshold)
        astart, astop = crossings(actual, threshold)
        out.update(
            reference_onset_seconds=rstart,
            generated_onset_seconds=astart,
            reference_offset_seconds=rstop,
            generated_offset_seconds=astop,
            offset_censored=bool(rstop is None or rstop >= times[-1] - 0.04),
        )
        out["onset_error_seconds"] = (
            None if rstart is None or astart is None else abs(astart - rstart)
        )
        out["offset_error_seconds"] = (
            None if out["offset_censored"] or astop is None else abs(astop - rstop)
        )
    else:
        out.update(
            onset_error_seconds=None, offset_error_seconds=None, offset_censored=True
        )
    return out


@torch.inference_mode()
def prepare(manifest_path, vae, device):
    rows, _, _, _, manifest = flow.spectrum.load_data(manifest_path)
    original = {r["id"]: r for r in manifest["rows"]}
    records, audit = [], []
    for row in rows:
        entry = original[row["id"]]
        position = np.genfromtxt(
            flow.spectrum.checked(entry["position"], manifest, manifest_path.parent),
            delimiter=",",
            names=True,
        )
        force = np.genfromtxt(
            flow.spectrum.checked(entry["force"], manifest, manifest_path.parent),
            delimiter=",",
            names=True,
        )
        wave, _ = sf.read(
            flow.spectrum.checked(entry["audio"], manifest, manifest_path.parent),
            always_2d=True,
            dtype="float32",
        )
        native = np.repeat(wave * flow.GAIN, 2, axis=1)
        padded = np.pad(native, ((0, (-len(native)) % HOP), (0, 0)))
        posterior = vae.encode(torch.tensor(padded.T[None], device=device)).latent_dist
        count = posterior.mean.shape[-1]
        times = (np.arange(count) + 0.5) * HOP / RATE
        velocity = position_speed(position, times)
        physical = trace_features(
            row["texture_id"], velocity, np.interp(times, force["time"], force["force"])
        )
        valid = len(native) // HOP
        if valid < flow.FRAMES:
            raise ValueError("insufficient unpadded target window")
        records.append(
            {
                "row": row,
                "mean": posterior.mean[0].cpu(),
                "std": posterior.std[0].cpu(),
                "physical": physical,
                "wave": native,
                "valid_frames": valid,
            }
        )
        if row["role"] == "train":
            env = envelope(native)
            tt = (np.arange(len(env)) + 0.5) * 0.02
            vv = position_speed(position, tt)
            lags = np.arange(-0.2, 0.201, 0.02)
            correlations = [
                float(np.corrcoef(env, position_speed(position, tt - lag))[0, 1])
                for lag in lags
            ]
            audit.append(
                {
                    "id": row["id"],
                    "source_duration_seconds": len(native) / RATE,
                    "position_end_seconds": float(position["time"][-1]),
                    "force_end_seconds": float(force["time"][-1]),
                    "travel_mm": float(position["Y"][0] - position["Y"][-1]),
                    "zero_lag_correlation": float(np.corrcoef(env, vv)[0, 1]),
                    "diagnostic_best_lag_seconds": float(lags[np.argmax(correlations)]),
                    "diagnostic_best_correlation": max(correlations),
                    "applied_shift_seconds": 0,
                    "velocity_proxy_max_mm_s": float(max(velocity)),
                }
            )
    return records, manifest, audit


def publish(output, name, native, frames):
    if native.ndim != 2 or native.shape[1] != 2 or not 0 < frames <= len(native):
        raise ValueError("invalid full event layout")
    full, _ = flow.codec.publish(output / f"{name}-full.wav", native, "FLOAT")
    row, pcm = flow.codec.publish(output / f"{name}.wav", native[:frames] * PLAYBACK)
    return {**row, "full": full, "playback_gain": PLAYBACK}, pcm


@torch.inference_mode()
def generate(model, vae, physical, seed):
    # Revalidate the full sequence before inference. No audio/source path input.
    texture = flow.spectrum.TEXTURES[int(np.argmax(physical[:3, 0]))]
    checked = trace_features(texture, physical[3] * 20 + 40, physical[4] * 0.25 + 0.75)
    if not np.allclose(checked, physical, rtol=0, atol=1e-6):
        raise ValueError("changing material or invalid feature encoding")
    latent = flow.sample_features(model, physical[None], seed, physical.shape[-1])
    return vae.decode(latent).sample[0].T.cpu().numpy()


@torch.inference_mode()
def evaluate(model, old, vae, records, output, report):
    all_preview, glass_preview = [], []
    for record in records:
        row, physical, real_wave = record["row"], record["physical"], record["wave"]
        n, count = len(real_wave), physical.shape[-1]
        real_entry, real = flow.codec.publish(
            output / f"{row['id']}-real.wav", real_wave * PLAYBACK
        )
        latent = record["mean"][None] + record["std"][None] * torch.randn(
            record["mean"][None].shape, generator=torch.Generator().manual_seed(314)
        )
        decoded = (
            vae.decode(latent.to(next(model.parameters()).device))
            .sample[0]
            .T.cpu()
            .numpy()
        )
        oracle_entry, oracle = publish(output, f"{row['id']}-codec", decoded, n)
        report["references"].append(
            {
                "id": row["id"],
                "real": real_entry,
                "codec": oracle_entry,
                "codec_metrics": event_metrics(
                    oracle, real, physical, row["commanded_speed_mm_s"]
                ),
            }
        )
        for seed in (314, 2718):
            native = generate(model, vae, physical, seed)
            delayed = np.concatenate(
                (physical[:, :1].repeat(5, axis=1), physical[:, :-5]), axis=1
            )
            delayed[3, :5] = -2  # Requested delay, not circular wrap or audio shift.
            wrong = generate(model, vae, delayed, seed)
            constant_features = flow.spectrum.features(
                row["texture_id"],
                row["commanded_speed_mm_s"],
                row["commanded_normal_force_N"],
            )[None]
            old_latent = flow.sample(old, constant_features, seed, frames=count)
            constant = vae.decode(old_latent).sample[0].T.cpu().numpy()
            envelope_gain = np.sqrt(
                np.clip(
                    np.interp(
                        (np.arange(len(constant)) + 0.5) / RATE,
                        (np.arange(count) + 0.5) * HOP / RATE,
                        physical[3] * 20 + 40,
                    )
                    / row["commanded_speed_mm_s"],
                    0,
                    2,
                )
            )
            variants = {
                "timed": native,
                "delayed_control": wrong,
                "constant": constant,
                "velocity_gate": constant * envelope_gain[:, None],
            }
            pcms = {}
            for name, wave in variants.items():
                entry, pcm = publish(output, f"{row['id']}-{name}-{seed}", wave, n)
                pcms[name] = pcm
                report["rows"].append(
                    {
                        "id": row["id"],
                        "role": row["role"],
                        "seed": seed,
                        "variant": name,
                        **entry,
                        **event_metrics(
                            pcm, real, physical, row["commanded_speed_mm_s"]
                        ),
                    }
                )
            if row["commanded_speed_mm_s"] == 40 and row["repeat"] == 0 and seed == 314:
                group = []
                for pcm in (
                    real,
                    pcms["timed"],
                    pcms["constant"],
                    pcms["velocity_gate"],
                    oracle,
                ):
                    group.extend((pcm, np.zeros((round(0.25 * RATE), 2))))
                all_preview.extend(group)
                if row["texture_id"] == 74 and row["commanded_normal_force_N"] == 0.5:
                    glass_preview.extend(group)
        print(
            json.dumps({"evaluated": row["id"], "rows": len(report["rows"])}),
            flush=True,
        )
        flow.save(output / "result.json", report)
    for name, waves in (
        ("comparison", all_preview),
        ("glass-comparison", glass_preview),
    ):
        report[name], _ = flow.codec.publish(
            output / f"{name}.wav", np.concatenate(waves)
        )
    report["comparison_order"] = (
        "wood/steel/glass,.5/1N,40mm/s,repeat0,seed314: real/timed/constant/velocity-gated/reference-aided-codec"
    )
    report["summary"] = []
    for role in ("train", "unseen_speed", "repeat_development"):
        for variant in ("timed", "delayed_control", "constant", "velocity_gate"):
            rr = [
                r
                for r in report["rows"]
                if r["role"] == role and r["variant"] == variant
            ]
            summary = {"role": role, "variant": variant, "count": len(rr)}
            for key in (
                "envelope_db_mae",
                "envelope_correlation",
                "pre_contrast_error_db",
                "post_contrast_error_db",
                "onset_error_seconds",
                "offset_error_seconds",
            ):
                values = [r[key] for r in rr if r[key] is not None]
                summary[key] = float(np.mean(values)) if values else None
                summary[key + "_count"] = len(values)
            report["summary"].append(summary)


def train(manifest_path, parent, output, device="cuda"):
    torch.set_num_threads(4)
    output = flow.fresh(output)
    shutil.copyfile(__file__, output / "executed-script.py")
    shutil.copyfile(Path(flow.__file__), output / "executed-flow.py")
    started = time.monotonic()
    report = {
        "status": "running",
        "scope": "nominal-clock full friction events; no physical timing or perceptual admission",
        "source_sha256": flow.codec.sha(manifest_path),
        "references": [],
        "rows": [],
    }
    flow.save(output / "result.json", report)
    try:
        model, parent_meta = flow.load_model(parent, device)
        old, _ = flow.load_model(parent, device)
        old.requires_grad_(False)
        if parent_meta["source_sha256"] != report["source_sha256"] or set(
            parent_meta["training_ids"]
        ) != {
            r["id"]
            for r in flow.spectrum.source.conditions(True)
            if flow.spectrum.role(r) == "train"
        }:
            raise ValueError("original24-TRAIN parent required")
        vae, codec_root = flow.load_codec(device)
        records, manifest, audit = prepare(manifest_path, vae, device)
        report["source_clock_audit"] = audit
        training = [r for r in records if r["row"]["role"] == "train"]
        torch.manual_seed(23)
        optimizer = torch.optim.AdamW(model.parameters(), lr=1e-3, weight_decay=1e-4)
        losses = []
        for step in range(2000):
            targets, features = [], []
            for r in training:
                last = r["valid_frames"] - flow.FRAMES
                first = (
                    0
                    if step % 3 == 0
                    else last
                    if step % 3 == 2
                    else int(torch.randint(last + 1, (1,)))
                )
                mean = r["mean"][:, first : first + flow.FRAMES]
                std = r["std"][:, first : first + flow.FRAMES]
                targets.append(mean + std * torch.randn_like(std))
                features.append(r["physical"][:, first : first + flow.FRAMES])
            target = (torch.stack(targets).to(device) - model.center) / model.scale
            physical = torch.tensor(np.stack(features), device=device)
            noise = torch.randn_like(target)
            t = torch.rand(len(target), device=device)
            mixed = noise * (1 - t[:, None, None]) + target * t[:, None, None]
            loss = (model(mixed, t, physical) - (target - noise)).square().mean()
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
                        {"step": step + 1, "loss200": float(np.mean(losses[-200:]))}
                    ),
                    flush=True,
                )
        model.eval()
        save_file(model.state_dict(), output / "model.safetensors")
        meta = {
            **parent_meta,
            "format": FORMAT,
            "parent_checkpoint_sha256": parent_meta["checkpoint_sha256"],
            "checkpoint_sha256": flow.codec.sha(output / "model.safetensors"),
            "playback_gain": PLAYBACK,
            "new_training_steps": 2000,
            "total_training_steps": 4000,
            "scope": report["scope"],
            "context_frames": 0,
            "output_frames": "variable32..256,full decode,not parent central-core extraction",
            "seed": 23,
            "time_conditioning": "published nominal clock,100ms average displacement speed and interpolated force; no audio alignment fit",
            "normalization": "unchanged parent TRAIN center/scale",
            "window_sampling": "cycle first/random/last32 valid unpadded frames per TRAIN record",
            "loss_windows200": [
                float(np.mean(losses[i : i + 200])) for i in range(0, 2000, 200)
            ],
            "source_license": manifest["license"],
        }
        flow.save(output / "model.json", meta)
        report.update(model=meta, training_seconds=time.monotonic() - started)
        for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
            shutil.copyfile(codec_root / name, output / name)
        flow.save(output / "result.json", report)
        evaluate(model, old, vae, records, output, report)
        physical, request = profile()
        wave = generate(model, vae, physical, 314)
        report["standalone_profile"], _ = publish(
            output,
            "requested-glass-event",
            wave,
            round(request["duration_seconds"] * RATE),
        )
        report["standalone_profile"].update(
            request=request, reference_audio_input=False, seed=314
        )
        report.update(status="complete", elapsed_seconds=time.monotonic() - started)
        flow.save(output / "result.json", report)
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        flow.save(
            output / "failure.json", {"status": "failed", "error": report["error"]}
        )
        flow.save(output / "result.json", report)
        raise
    print(
        json.dumps(
            {
                "status": "complete",
                "primary_artifact": report["standalone_profile"]["wav"],
            }
        ),
        flush=True,
    )


@torch.inference_mode()
def evaluate_saved(manifest_path, parent, directory, output, device="cuda"):
    torch.set_num_threads(4)
    output = flow.fresh(output)
    shutil.copyfile(__file__, output / "executed-script.py")
    shutil.copyfile(Path(flow.__file__), output / "executed-flow.py")
    started = time.monotonic()
    model, meta = flow.load_model(directory, device, expected_format=FORMAT)
    old, parent_meta = flow.load_model(parent, device)
    old.requires_grad_(False)
    if meta["parent_checkpoint_sha256"] != parent_meta["checkpoint_sha256"] or meta[
        "source_sha256"
    ] != flow.codec.sha(manifest_path):
        raise ValueError("saved model parent/source mismatch")
    vae, root = flow.load_codec(device)
    report = {
        "status": "evaluating",
        "new_training_steps": 0,
        "model": meta,
        "model_directory": str(directory),
        "source_sha256": flow.codec.sha(manifest_path),
        "original_report_sha256": flow.codec.sha(directory / "result.json"),
        "effective_output": "full variable-length event; original inherited context_frames field is unused",
        "references": [],
        "rows": [],
    }
    flow.save(output / "result.json", report)
    try:
        records, _, audit = prepare(manifest_path, vae, device)
        report["source_clock_audit"] = audit
        for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
            shutil.copyfile(root / name, output / name)
        evaluate(model, old, vae, records, output, report)
        report.update(status="complete", elapsed_seconds=time.monotonic() - started)
        flow.save(output / "result.json", report)
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        flow.save(
            output / "failure.json", {"status": "failed", "error": report["error"]}
        )
        flow.save(output / "result.json", report)
        raise
    print(
        json.dumps({"status": "complete", "comparison": report["glass-comparison"]}),
        flush=True,
    )


@torch.inference_mode()
def render(directory, output, texture, speed, force, seed, device="cuda"):
    torch.set_num_threads(4)
    physical, request = profile(texture, speed, force)
    model, meta = flow.load_model(directory, device, expected_format=FORMAT)
    vae, root = flow.load_codec(device)
    wave = generate(model, vae, physical, seed)
    output = flow.fresh(output)
    result, _ = publish(
        output, "generated", wave, round(request["duration_seconds"] * RATE)
    )
    for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
        shutil.copyfile(root / name, output / name)
    result.update(
        reference_audio_input=False,
        request=request,
        seed=seed,
        model_sha256=meta["checkpoint_sha256"],
    )
    flow.save(output / "result.json", result)
    return result


def noise_floor_countercheck(result_path, output):
    """Posthoc TRAIN-only quiet floor; fixed existing PCM, no generator changes."""
    report = json.loads(result_path.read_text())
    if report["status"] != "complete" or report["model"]["format"] != FORMAT:
        raise ValueError("complete event evaluation required")
    output = flow.fresh(output)

    def read(entry):
        path = Path(entry["wav"]).resolve()
        if (
            not path.is_relative_to(result_path.parent.resolve())
            or path.stat().st_size > 4_000_000
            or flow.codec.sha(path) != entry["sha256"]
        ):
            raise ValueError("invalid fixed PCM identity")
        wave, rate = sf.read(path, always_2d=True)
        if rate != RATE or wave.shape[1] != 2 or not np.isfinite(wave).all():
            raise ValueError("invalid event PCM")
        return wave

    references = {r["id"]: read(r["real"]) for r in report["references"]}
    training = set(report["model"]["training_ids"])
    if training != {
        r["id"]
        for r in flow.spectrum.source.conditions(True)
        if flow.spectrum.role(r) == "train"
    }:
        raise ValueError("original TRAIN quiet evidence required")
    # First100ms precedes the observed TRAIN motion. One scalar, all materials,
    # loads and seeds; not a per-case noise fit or a claim this noise is contact.
    variance = float(
        np.mean(
            [
                np.mean(
                    resample_poly(references[ident][: round(0.1 * RATE)].mean(1), 1, 2)
                    ** 2
                )
                for ident in sorted(training)
            ]
        )
    )
    rows = []
    preview = []
    timed = {(r["id"], r["seed"]): r for r in report["rows"] if r["variant"] == "timed"}
    for row in report["rows"]:
        if row["variant"] != "velocity_gate":
            continue
        gate = read(row)
        real = references[row["id"]]
        noise = np.random.default_rng(row["seed"] + 17).normal(
            0, np.sqrt(variance), int(np.ceil(len(gate) / 2))
        )
        noise = resample_poly(noise, 2, 1)[: len(gate)]
        candidate = gate + noise[:, None]
        entry, pcm = flow.codec.publish(
            output / f"{row['id']}-floor-{row['seed']}.wav", candidate
        )
        delta = float(np.mean(np.abs(envelope(pcm) - envelope(real))))
        rows.append(
            {
                "id": row["id"],
                "role": row["role"],
                "seed": row["seed"],
                **entry,
                "floor_envelope_db_mae": delta,
                "timed_envelope_db_mae": timed[row["id"], row["seed"]][
                    "envelope_db_mae"
                ],
                "silent_gate_envelope_db_mae": row["envelope_db_mae"],
            }
        )
        if row["id"] == "74_0_40_500_0" and row["seed"] == 314:
            for wave in (real, read(timed[row["id"], 314]), gate, pcm):
                preview.extend((wave, np.zeros((round(0.25 * RATE), 2))))
    comparison, _ = flow.codec.publish(
        output / "comparison.wav", np.concatenate(preview)
    )
    summary = []
    for role in ("train", "unseen_speed", "repeat_development"):
        rr = [r for r in rows if r["role"] == role]
        summary.append(
            {
                "role": role,
                "count": len(rr),
                "timed_wins": sum(
                    r["timed_envelope_db_mae"] < r["floor_envelope_db_mae"] for r in rr
                ),
                **{
                    key: float(np.mean([r[key] for r in rr]))
                    for key in (
                        "floor_envelope_db_mae",
                        "timed_envelope_db_mae",
                        "silent_gate_envelope_db_mae",
                    )
                },
            }
        )
    result = {
        "status": "complete",
        "scope": "posthoc recorded-background countercheck,not a learned model or physical-quality judge",
        "source_report_sha256": flow.codec.sha(result_path),
        "training_ids": sorted(training),
        "noise_rms22k_published_scale": float(np.sqrt(variance)),
        "noise_seed_rule": "existing seed+17; no seed selection",
        "comparison_order": "glass40mm/s,.5N,repeat0,seed314: real/timed/silent-velocity-gate/gate-plus-TRAIN-background",
        "rows": rows,
        "summary": summary,
        "comparison": comparison,
    }
    flow.save(output / "result.json", result)
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--corpus", type=Path)
    group.add_argument("--model", type=Path)
    parser.add_argument("--parent", type=Path)
    parser.add_argument("--evaluate-model", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--texture", type=int, default=74)
    parser.add_argument("--speed", type=float, default=40)
    parser.add_argument("--force", type=float, default=0.5)
    parser.add_argument("--seed", type=int, default=314)
    args = parser.parse_args()
    if args.evaluate_model and not args.corpus:
        parser.error("--evaluate-model requires --corpus and --parent")
    if args.corpus:
        if args.parent is None:
            parser.error("--corpus requires --parent")
        if args.evaluate_model:
            evaluate_saved(
                args.corpus.resolve(),
                args.parent.resolve(),
                args.evaluate_model.resolve(),
                args.output,
            )
        else:
            train(args.corpus.resolve(), args.parent.resolve(), args.output)
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
