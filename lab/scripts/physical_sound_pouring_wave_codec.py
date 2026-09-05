"""Frozen waveform-codec control on disclosed pouring crops.

Powered by Stability AI; TangoFlux / Hung et al. Local research only. This
reconstructs recordings; it is not object-conditioned sound generation.
"""

import argparse
import hashlib
import shutil
from pathlib import Path

import numpy as np
import physical_sound_pouring_compare as compare
import physical_sound_pouring_cvae as v
import physical_sound_tangoflux_pilot as tango
import soundfile as sf
import torch
from scipy.signal import resample_poly

CODEC_SHA = "d73619a1d1e1dc48e606632931ffce440b4959ce2a4ed5a3522c3bb573b103be"
NATIVE_SAMPLES = round(v.p.SAMPLES * 44100 / v.p.RATE)
LATENT_FRAMES = (NATIVE_SAMPLES + 2047) // 2048


def load_codec(device="cuda"):
    from diffusers import AutoencoderOobleck
    from huggingface_hub import snapshot_download
    from safetensors.torch import load_file

    root = Path(
        snapshot_download(tango.MODEL, revision=tango.REVISION, local_files_only=True)
    )
    checkpoint = root / "vae.safetensors"
    if (
        checkpoint.stat().st_size != 624490208
        or hashlib.sha256(checkpoint.read_bytes()).hexdigest() != CODEC_SHA
    ):
        raise ValueError("codec identity mismatch")
    codec = AutoencoderOobleck().eval().requires_grad_(False)
    codec.load_state_dict(load_file(checkpoint), strict=True)
    return codec.to(device), root


def input_wave(wave):
    if (
        wave.shape != (v.p.SAMPLES,)
        or not np.isfinite(wave).all()
        or abs(wave).max() > 0.98
    ):
        raise ValueError("expected bounded 4.08s mono16k waveform")
    native = resample_poly(wave, 441, 160).astype(np.float32)
    native = np.pad(native, (0, LATENT_FRAMES * 2048 - len(native)))
    return np.stack([native, native])


def mono_wave(native):
    if (
        native.shape != (2, LATENT_FRAMES * 2048)
        or not np.isfinite(native).all()
        or abs(native).max() > 0.98
    ):
        raise ValueError("unsafe codec output; no normalization")
    return resample_poly(native[:, :NATIVE_SAMPLES].mean(0), 160, 441).astype(
        np.float32
    )[: v.p.SAMPLES]


@torch.inference_mode()
def probe(source, output, device="cuda"):
    torch.set_num_threads(4)
    rows, provenance = v.p.load_source(source)
    codec, model_root = load_codec(device)
    output = v.phase.d.fresh_output(output)
    for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
        shutil.copyfile(model_root / name, output / name)
    selected = {"train-first": next(r for r in rows if r["role"] == "train")}
    for row in rows:
        if row["role"] == "unseen_container":
            selected.setdefault(row["container_id"], row)
    records, preview = [], []
    for name, row in selected.items():
        for part in ("first", "middle"):
            _, controls, real, offset = compare.phase_crop(row, part)
            posterior = codec.encode(
                torch.tensor(input_wave(real)[None], device=device)
            ).latent_dist
            pending = [("real", None, real, None)]
            for kind, seed in [
                ("mean", None),
                ("sample", 314),
                ("sample", 2718),
                ("sample", 1618),
            ]:
                latent = (
                    posterior.mean
                    if seed is None
                    else posterior.sample(
                        generator=torch.Generator(device=device).manual_seed(seed)
                    )
                )
                native = codec.decode(latent).sample[0].cpu().numpy()
                pending.append((kind, seed, mono_wave(native), native))
            for kind, seed, wave, native in pending:
                key = f"{name}-{part}-{kind}-{seed}"
                record = {
                    "object": name,
                    "item_id": row["item_id"],
                    "phase": part,
                    "start_sample16k": offset,
                    "controls": controls.tolist(),
                    "kind": kind,
                    "seed": seed,
                    "reference_audio_input": True,
                    **v.h.save_wav(output / (key + ".wav"), wave),
                    **v.p.metrics(wave, real),
                }
                if native is not None:
                    path = output / (key + "-stereo.wav")
                    sf.write(
                        path, native[:, :NATIVE_SAMPLES].T, 44100, subtype="PCM_16"
                    )
                    record["stereo"] = {
                        "wav": str(path),
                        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                        "gain": 1,
                    }
                records.append(record)
                if part == "middle" and seed in (None, 2718):
                    preview.extend([wave, np.zeros(v.p.RATE // 2)])
        print({"codec_evaluated": name}, flush=True)
    v.p.save_json(
        output / "result.json",
        {
            "status": "complete",
            "scope": "frozen codec reconstruction, not reference-free generation; 16k bandlimited input resampled to44.1k dual mono",
            "codec_sha256": CODEC_SHA,
            "model": tango.MODEL,
            "revision": tango.REVISION,
            "terms": provenance["terms"],
            "source_sha256": hashlib.sha256(
                (source / "source.json").read_bytes()
            ).hexdigest(),
            "rows": records,
            "comparison": {
                **v.h.save_wav(output / "comparison.wav", np.concatenate(preview)),
                "order": "train-first/glass18/PET30 middle; real/mean/sample2718; gain1",
            },
        },
    )
    v.p.save_json(
        output / "tag-input.json",
        {
            "status": "complete",
            "seconds": 4.08,
            "cases": [
                {"id": f"{i}-{r['kind']}", "diagnostic_id": "water-pour"}
                for i, r in enumerate(records)
            ],
            "rows": [{**r, "case": i} for i, r in enumerate(records)],
            "controls": [],
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--device", choices=("cuda", "cpu"), default="cuda")
    args = parser.parse_args()
    probe(args.source, args.output, args.device)
