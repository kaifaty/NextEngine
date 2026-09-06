"""Spectral-detail/phase oracle and separately labelled source-free refinement.

Oracle mode uses the first training recording; refinement mode generates from
conditions and evaluates disclosed development records. No fitting or promotion.
"""

import argparse
import hashlib
from pathlib import Path

import numpy as np
import physical_sound_pouring_temporal_decoder as d
import torch
from torch.nn import functional as F

MODES = ("noise_phase", "iterative_phase", "coarse65", "expected_noise_gain")


def transform(wave):
    return torch.stft(
        wave,
        d.FFT,
        d.p.HOP,
        window=torch.hann_window(d.FFT, device=wave.device),
        return_complex=True,
    )


def inverse(spectrum):
    return torch.istft(
        spectrum,
        d.FFT,
        d.p.HOP,
        window=torch.hann_window(d.FFT, device=spectrum.device),
        length=d.p.SAMPLES,
    )


def coarse(magnitude):
    bands = F.interpolate(
        magnitude.clamp_min(1e-8).log().T[:, None],
        size=65,
        mode="linear",
        align_corners=True,
    )
    return F.interpolate(bands, size=d.FFT // 2 + 1, mode="linear", align_corners=True)[
        :, 0
    ].T.exp()


@torch.inference_mode()
def resynthesize(wave, seed, mode):
    if (
        wave.shape != (d.p.SAMPLES,)
        or not torch.isfinite(wave).all()
        or not isinstance(seed, int)
        or not 0 <= seed < 2**32
        or mode not in MODES
    ):
        raise ValueError("invalid oracle input")
    magnitude = transform(wave).abs()
    noise = torch.randn(
        wave.shape,
        generator=torch.Generator(device=wave.device).manual_seed(seed),
        device=wave.device,
    )
    spectrum = transform(noise)
    if mode == "expected_noise_gain":
        # E|STFT(white noise)|² = sum(window²); no inversion of this noise draw.
        gain = (
            magnitude
            / torch.hann_window(d.FFT, device=wave.device).square().sum().sqrt()
        )
        return inverse(spectrum * gain)
    if mode == "coarse65":
        magnitude = coarse(magnitude)
    phase = spectrum / spectrum.abs().clamp_min(1e-12)
    for _ in range(32 if mode == "iterative_phase" else 0):
        updated = transform(inverse(magnitude * phase))
        phase = updated / updated.abs().clamp_min(1e-12)
    return inverse(magnitude * phase)


@torch.inference_mode()
def refine_generated(model, controls, seed, iterations=32):
    """Refine the network's generated spectrogram; no target recording input."""
    if not isinstance(seed, int) or not 0 <= seed < 2**32 or iterations not in (0, 32):
        raise ValueError("invalid refinement request")
    device = next(model.parameters()).device
    c = torch.tensor(d.h.grid(controls, d.TIMES)[None], device=device)
    hz = model.frequency(c)
    if not torch.isfinite(hz).all() or (hz < 50).any() or (hz > 7800).any():
        raise ValueError("invalid generated frequency")
    noise = torch.randn(
        (1, d.p.SAMPLES),
        generator=torch.Generator(device=device).manual_seed(seed),
        device=device,
    )
    spectrum = transform(noise[0]) * model.response(c, hz)[0]
    magnitude = spectrum.abs()
    phase = spectrum / magnitude.clamp_min(1e-12)
    for _ in range(iterations):
        updated = transform(inverse(magnitude * phase))
        phase = updated / updated.abs().clamp_min(1e-12)
    result = inverse(magnitude * phase).cpu().numpy()
    if not np.isfinite(result).all() or abs(result).max() > 0.98:
        raise ValueError("unsafe refined waveform; no clipping or attenuation")
    return result


def run_generated(source, checkpoint, output, device="cuda"):
    torch.set_num_threads(4)
    model, meta = d.load(checkpoint, device)
    rows, _ = d.p.load_source(source)
    if (
        meta["train_ids"] != [r["item_id"] for r in rows if r["role"] == "train"]
        or meta["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    ):
        raise ValueError("refinement requires full trained decoder/source identity")
    output = d.fresh_output(output)
    report = {
        "status": "running",
        "scope": "source-free refinement of rejected temporal decoder; references only for evaluation",
        "checkpoint_sha256": meta["checkpoint_sha256"],
        "rows": [],
        "novel": [],
    }
    preview = []
    for name, height, material, duration in [
        ("glass10", 10, "glass", 15),
        ("glass16", 16, "glass", 15),
        ("pet10", 10, "plastic_pet", 15),
        ("glass10fast", 10, "glass", 8),
    ]:
        c = d.controls_for(height, material, duration)
        for seed in (314, 2718, 1618):
            for kind, steps in (("native", 0), ("refined", 32)):
                wave = refine_generated(model, c, seed, steps)
                report["novel"].append(
                    {
                        "profile": name,
                        "controls": c.tolist(),
                        "kind": kind,
                        "seed": seed,
                        **d.h.save_wav(output / f"{name}-{kind}-{seed}.wav", wave),
                    }
                )
                if seed == 2718:
                    preview.extend([wave, np.zeros(d.p.RATE // 2)])
        print({"profile": name, "reference_audio_input": False}, flush=True)
    selected = {}
    for row in rows:
        if row["role"] == "unseen_container":
            selected.setdefault(row["container_id"], row)
    for i, row in enumerate(selected.values()):
        for phase in ("first", "middle"):
            _, c, real, _ = d.compare.phase_crop(row, phase)
            pending = [("real", None, real)]
            for seed in (314, 2718, 1618):
                pending.extend(
                    (kind, seed, refine_generated(model, c, seed, steps))
                    for kind, steps in (("native", 0), ("refined", 32))
                )
            for kind, seed, wave in pending:
                report["rows"].append(
                    {
                        "item_id": row["item_id"],
                        "phase": phase,
                        "kind": kind,
                        "seed": seed,
                        **d.h.save_wav(
                            output / f"dev-{i}-{phase}-{kind}-{seed}.wav", wave
                        ),
                        **d.p.metrics(wave, real),
                    }
                )
    report["comparison"] = {
        **d.h.save_wav(output / "comparison.wav", np.concatenate(preview)),
        "order": "glass10/glass16/PET10/glass10fast; native/refined; seed2718; gain1",
    }
    report["status"] = "complete"
    d.p.save_json(output / "result.json", report)
    inputs = report["rows"] + report["novel"]
    d.p.save_json(
        output / "tag-input.json",
        {
            "status": "complete",
            "seconds": 4.08,
            "cases": [
                {"id": f"{i}-{r['kind']}", "diagnostic_id": "water-pour"}
                for i, r in enumerate(inputs)
            ],
            "rows": [{**r, "case": i} for i, r in enumerate(inputs)],
            "controls": [],
        },
    )


def run(source, fitted, output, device="cpu"):
    torch.set_num_threads(4)
    source_rows, _ = d.p.load_source(source)
    row = next(r for r in source_rows if r["role"] == "train")
    model, meta = d.load(fitted, device)
    if (
        meta["train_ids"] != [row["item_id"]]
        or meta["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    ):
        raise ValueError("expected the single-record diagnostic checkpoint")
    output = d.fresh_output(output)
    records, preview = [], []
    for phase_name in ("first", "middle"):
        _, controls, real, _ = d.compare.phase_crop(row, phase_name)
        wave = torch.tensor(real, device=device)
        pending = [("real", None, real)]
        for seed in (314, 2718, 1618):
            pending.append(("learned", seed, d.sample(model, controls, seed)))
            pending.extend(
                (mode, seed, resynthesize(wave, seed, mode).cpu().numpy())
                for mode in MODES
            )
        for kind, seed, sound in pending:
            records.append(
                {
                    "kind": kind,
                    "phase": phase_name,
                    "seed": seed,
                    **d.h.save_wav(output / f"{phase_name}-{kind}-{seed}.wav", sound),
                    **d.p.metrics(sound, real),
                }
            )
            if seed in (None, 2718):
                preview.extend([sound, np.zeros(d.p.RATE // 2)])
    report = {
        "status": "complete",
        "scope": "source-dependent reconstruction of first TRAINING record; not reference-free, physical validation or new conditions",
        "item_id": row["item_id"],
        "container_id": row["container_id"],
        "source_sha256": meta["source_sha256"],
        "fitted_checkpoint_sha256": meta["checkpoint_sha256"],
        "rows": records,
        "comparison": {
            **d.h.save_wav(output / "comparison.wav", np.concatenate(preview)),
            "order": "first/middle; real/learned/noise_phase/iterative_phase/coarse65/expected_noise_gain; seed2718; gain1",
        },
    }
    d.p.save_json(output / "result.json", report)
    d.p.save_json(
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
    print({"diagnostic_audio": report["comparison"]["wav"]}, flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--fitted", type=Path)
    mode.add_argument("--refine-model", type=Path)
    parser.add_argument("--device", choices=("cpu", "cuda"), default="cpu")
    args = parser.parse_args()
    if args.refine_model:
        run_generated(args.source, args.refine_model, args.output, args.device)
    else:
        run(args.source, args.fitted, args.output, args.device)
