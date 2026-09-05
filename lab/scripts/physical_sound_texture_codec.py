"""Frozen Oobleck feasibility on disclosed, controlled friction recordings.

Reference-aided codec diagnosis, NOT source-free synthesis or quality admission.
All channels share a TRAIN-derived gain; no fitting, per-item normalization,
event selection, hidden test access, or generator/prompt tuning.
"""

from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import shutil
import time
from pathlib import Path

import numpy as np
import physical_sound_tangoflux_pilot as tango
import physical_sound_texture_fit as fit
import soundfile as sf
import torch
from scipy.signal import resample_poly

RATE = 44100
CHANNELS = ("clean", "main", "machine")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def level(wave):
    return float(10 * np.log10(max(np.mean(np.asarray(wave, np.float64) ** 2), 1e-18)))


def shared_gain(waves):
    if not waves or any(w.ndim != 1 or not np.isfinite(w).all() for w in waves):
        raise ValueError("finite mono TRAIN signals required")
    peak = max(float(np.max(np.abs(w))) for w in waves)
    if not 0 < peak <= 1:
        raise ValueError("invalid TRAIN peak")
    return min(100.0, 0.5 / peak)


def crop(wave, start):
    first, count = round(start * RATE), round(0.75 * RATE)
    if wave.ndim != 2 or wave.shape[1] != 2 or first < 0:
        raise ValueError("expected time-major stereo and nonnegative start")
    out = wave[first : first + count]
    if len(out) != count or not np.isfinite(out).all():
        raise ValueError("incomplete/nonfinite aligned crop")
    return out


def publish(path, wave, subtype="PCM_24"):
    if wave.ndim != 2 or wave.shape[1] != 2 or not np.isfinite(wave).all():
        raise ValueError("expected finite time-major stereo")
    if subtype != "FLOAT" and np.max(np.abs(wave)) > 0.98:
        raise ValueError("PCM headroom exceeded; do not normalize individual outputs")
    sf.write(path, wave, RATE, subtype=subtype)
    pcm, rate = sf.read(path, always_2d=True)
    if rate != RATE or pcm.shape != wave.shape or not np.isfinite(pcm).all():
        raise ValueError("published PCM mismatch")
    return {
        "wav": str(path),
        "sha256": sha(path),
        "frames": len(pcm),
        "subtype": subtype,
        "peak": float(np.max(np.abs(pcm))),
        "level_dbfs": level(pcm),
        "mono_level_dbfs": level(pcm.mean(axis=1)),
    }, pcm


def metrics(candidate, reference):
    # Evaluate the published PCM, on the exact source-aligned interval. No onset
    # selection, loudness matching or shifting to improve reconstruction scores.
    cand = resample_poly(candidate.mean(axis=1), 1, 2)
    real = resample_poly(reference.mean(axis=1), 1, 2)
    delta = fit.spectrum(cand)[1:] - fit.spectrum(real)[1:]
    return {
        **fit.waveform_metrics(cand, real),
        "shape_rmse_db": float(np.sqrt(np.mean((delta - delta.mean()) ** 2))),
        "stereo_level_error_db": level(candidate) - level(reference),
    }


def response_pairs(rows):
    expected = {
        r["id"]: r for r in fit.source.conditions(True) if fit.role(r) == "train"
    }
    if len(rows) != 24 or {r["id"] for r in rows} != set(expected):
        raise ValueError("exact original 24 TRAIN conditions required")
    by_key = {}
    for row in rows:
        if any(row[key] != value for key, value in expected[row["id"]].items()):
            raise ValueError("changed TRAIN condition metadata")
        key = (
            row["texture_id"],
            row["commanded_speed_mm_s"],
            row["commanded_normal_force_N"],
        )
        by_key[key] = row["id"]
    pairs = []
    speeds = (20, 30, 50, 60)
    for texture in fit.TEXTURES:
        for force in (0.5, 1.0):
            for low, high in itertools.pairwise(speeds):
                pairs.append(
                    ("speed", by_key[texture, low, force], by_key[texture, high, force])
                )
        for speed in speeds:
            pairs.append(
                ("load", by_key[texture, speed, 0.5], by_key[texture, speed, 1.0])
            )
    return pairs


def summarize(rows, records):
    pairs = response_pairs(rows)
    index = {(r["id"], r["channel"], r["arm"], r["variant"]): r for r in records}
    result = []
    for channel in CHANNELS:
        for arm in ("native", "shared"):
            for variant in ("mean", "sample314"):
                for axis in ("speed", "load"):
                    values = []
                    for kind, low, high in pairs:
                        if kind != axis:
                            continue

                        real, decoded = (
                            index[high, channel, arm, v]["level_dbfs"]
                            - index[low, channel, arm, v]["level_dbfs"]
                            for v in ("real", variant)
                        )
                        values.append(
                            {
                                "low": low,
                                "high": high,
                                "real_delta_db": real,
                                "codec_delta_db": decoded,
                                "error_db": decoded - real,
                            }
                        )
                    errors = [abs(v["error_db"]) for v in values]
                    result.append(
                        {
                            "channel": channel,
                            "arm": arm,
                            "variant": variant,
                            "axis": axis,
                            "count": len(values),
                            "same_direction": sum(
                                int(
                                    np.sign(v["real_delta_db"])
                                    == np.sign(v["codec_delta_db"])
                                )
                                for v in values
                            ),
                            "delta_mae_db": float(np.mean(errors)),
                            "pairs": values,
                        }
                    )
    return result


def run(manifest_path, output):
    from diffusers import AutoencoderOobleck
    from huggingface_hub import snapshot_download
    from safetensors.torch import load_file

    manifest_path, output = manifest_path.resolve(), output.resolve()
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("new external output directory required")
    # Existing loader verifies the full already-open grid; codec execution below
    # uses only repeat0 at20/30/50/60. No roles are promoted to pristine evidence.
    all_rows, _, _, _, manifest = fit.load_data(manifest_path)
    rows = [r for r in all_rows if r["role"] == "train"]
    response_pairs(rows)
    originals = {r["id"]: r for r in manifest["rows"]}
    signals = {}
    for row in rows:
        original = originals[row["id"]]
        clean, _ = sf.read(
            fit.checked(original["audio"], manifest, manifest_path.parent),
            always_2d=True,
        )
        raw, _ = sf.read(
            fit.checked(original["raw_audio"], manifest, manifest_path.parent),
            always_2d=True,
        )
        for channel, wave in zip(
            CHANNELS, (clean[:, 0], raw[:, 0], raw[:, 1]), strict=True
        ):
            signals[row["id"], channel] = wave.astype(np.float32)
    gain = shared_gain(list(signals.values()))
    output.mkdir(parents=True)
    shutil.copyfile(__file__, output / "executed-script.py")
    started = time.monotonic()
    report = {
        "status": "running",
        "reference_audio_input": True,
        "new_training_steps": 0,
        "scope": "disclosed TRAIN codec feasibility; neither source-free generation nor quality admission",
        "source_manifest": str(manifest_path),
        "source_sha256": sha(manifest_path),
        "source_article": manifest["article"],
        "source_license": manifest["license"],
        "source_authors": manifest["authors"],
        "model": tango.MODEL,
        "revision": tango.REVISION,
        "attribution": "Powered by Stability AI; TangoFlux / Hung et al.; Cluster Haptic Texture Dataset / Eguchi et al.",
        "arms": {"native": 1.0, "shared": gain},
        "gain_rule": "min(100,0.5/max full-recording absolute peak across all24 TRAIN and3channels); no output gain",
        "channel_semantics": {
            "clean": "published noise-cancelled mono",
            "main": "raw microphone channel0",
            "machine": "raw machine-reference microphone channel1, not an independent quality judge",
        },
        "window": "full native recording encoded, right zero-pad to codec stride; score same central position-aligned0.75s",
        "conditions": rows,
        "rows": [],
        "silence_controls": [],
    }

    def save():
        (output / "result.json").write_text(
            json.dumps(report, indent=2, allow_nan=False) + "\n"
        )

    save()
    try:
        torch.set_num_threads(4)
        source = Path(
            snapshot_download(
                tango.MODEL, revision=tango.REVISION, local_files_only=True
            )
        )
        vae = AutoencoderOobleck().eval().requires_grad_(False)
        vae.load_state_dict(load_file(source / "vae.safetensors"), strict=True)
        vae.to("cuda")
        report["vae_sha256"] = sha(source / "vae.safetensors")
        report["runtime"] = {
            "torch": torch.__version__,
            "dtype": "float32",
            "codec_stride": vae.hop_length,
        }
        for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
            shutil.copyfile(source / name, output / name)
        previews = []
        with torch.no_grad():
            for row in rows + [{"id": "silence", "crop_start_seconds": 1.0}]:
                is_silence = row["id"] == "silence"
                for channel in ("silence",) if is_silence else CHANNELS:
                    wave = (
                        np.zeros(3 * RATE, np.float32)
                        if is_silence
                        else signals[row["id"], channel]
                    )
                    for arm, input_gain in (
                        (("native", 1.0),) if is_silence else report["arms"].items()
                    ):
                        stereo = np.repeat((wave * input_gain)[:, None], 2, axis=1)
                        real_crop = crop(stereo, row["crop_start_seconds"])
                        name = f"{row['id']}-{channel}-{arm}"
                        entry, real_pcm = publish(
                            output / f"{name}-real.wav", real_crop
                        )
                        entries = [
                            {
                                "id": row["id"],
                                "channel": channel,
                                "arm": arm,
                                "variant": "real",
                                **entry,
                            }
                        ]
                        length = len(stereo)
                        padded = np.pad(
                            stereo, ((0, (-length) % vae.hop_length), (0, 0))
                        )
                        posterior = vae.encode(
                            torch.from_numpy(padded.T.copy())[None].to("cuda")
                        ).latent_dist
                        selected = [real_pcm]
                        for variant in ("mean", "sample314"):
                            rng = torch.Generator(device="cuda").manual_seed(314)
                            latent = (
                                posterior.mean
                                if variant == "mean"
                                else posterior.sample(generator=rng)
                            )
                            decoded = vae.decode(latent).sample[0].T.cpu().numpy()
                            if len(decoded) != len(padded):
                                raise ValueError("codec alignment/length changed")
                            full, _ = publish(
                                output / f"{name}-{variant}-full.wav", decoded, "FLOAT"
                            )
                            entry, pcm = publish(
                                output / f"{name}-{variant}.wav",
                                crop(decoded, row["crop_start_seconds"]),
                            )
                            entries.append(
                                {
                                    "id": row["id"],
                                    "channel": channel,
                                    "arm": arm,
                                    "variant": variant,
                                    "input_gain": input_gain,
                                    "original_frames": length,
                                    "full": full,
                                    **entry,
                                    **metrics(pcm, real_pcm),
                                }
                            )
                            selected.append(pcm)
                        (
                            report["silence_controls"] if is_silence else report["rows"]
                        ).extend(entries)
                        if (
                            not is_silence
                            and channel == "clean"
                            and arm == "shared"
                            and row["commanded_speed_mm_s"] in (20, 60)
                        ):
                            for pcm in selected:
                                previews.extend(
                                    (pcm, np.zeros((round(0.25 * RATE), 2)))
                                )
                        save()
                        print(
                            json.dumps(
                                {
                                    "condition": name,
                                    "completed": len(report["rows"]),
                                    "gain": input_gain,
                                }
                            ),
                            flush=True,
                        )
        report["paired_responses"] = summarize(rows, report["rows"])
        report["comparison"], _ = publish(
            output / "comparison.wav", np.concatenate(previews)
        )
        report["comparison_order"] = (
            "source order: wood/steel/glass,20/60mm/s,.5/1N; each real/mean/sample314; clean shared-gain arm only"
        )
        report.update(status="complete", elapsed_seconds=time.monotonic() - started)
        save()
        print(
            json.dumps({"status": "complete", "comparison": report["comparison"]}),
            flush=True,
        )
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        save()
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    run(args.corpus, args.output)
