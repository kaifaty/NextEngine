"""Learn relative phase-level calibration around the frozen neural texture.

Only a global phase slope is added, not material/geometry calibration. Internet
recording gain cancels in paired targets. No recording is needed at inference.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import physical_sound_pouring_compare as compare
import physical_sound_pouring_pilot as p
import soundfile as sf
import torch


def level(wave):
    return 20 * np.log10(max(float(np.sqrt(np.mean(wave**2))), 1e-8))


def coefficients(controls, levels):
    x = np.c_[np.ones(len(controls) * 2), controls.reshape(-1, 11)].astype(np.float64)
    y = levels.reshape(-1).astype(np.float64)
    cx = (x.reshape(-1, 2, 12) - x.reshape(-1, 2, 12).mean(1, keepdims=True)).reshape(
        -1, 12
    )
    cy = (levels - levels.mean(1, keepdims=True)).reshape(-1)
    penalty = np.eye(12) * 0.01
    penalty[0, 0] = 0

    def solve(a, b):
        return np.linalg.lstsq(a.T @ a + penalty, a.T @ b, rcond=None)[0]

    signs = np.random.default_rng(53).choice([-1, 1], size=(len(levels), 1))
    return {
        "absolute": solve(x, y),
        "relative": solve(cx, cy),
        "shuffled": solve(cx, (cy.reshape(-1, 2) * signs).reshape(-1)),
        "zero-phase": np.zeros(12),
    }


def calibrate(wave, anchor_wave, progress, slope):
    if (
        not np.isfinite([progress, slope]).all()
        or not 0 <= progress <= 1
        or not -100 < slope < 100
    ):
        raise ValueError("invalid phase calibration")
    if (
        wave.shape != anchor_wave.shape
        or wave.shape != (p.SAMPLES,)
        or not np.isfinite(wave).all()
        or not np.isfinite(anchor_wave).all()
    ):
        raise ValueError("invalid neural waveform")
    gain = 10 ** ((level(anchor_wave) + slope * progress - level(wave)) / 20)
    result = (wave * gain).astype(np.float32)
    if not np.isfinite(result).all() or abs(result).max() > 0.98:
        raise ValueError("calibration exceeds headroom; no automatic attenuation")
    return result, float(gain)


def render(
    parent, calibration, output, controls, seed=2718, device="cpu", audition_gain=1
):
    """Run both learned artifacts without opening a dataset or target recording."""
    parent, calibration, output = (x.resolve() for x in (parent, calibration, output))
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("fresh external output required")
    if (
        controls.shape != (11,)
        or not np.isfinite(controls).all()
        or not 0 <= seed < 2**32
        or not np.isfinite(audition_gain)
        or not 0 < audition_gain <= 100
    ):
        raise ValueError("invalid controls or seed")
    path = calibration / "model.json"
    if path.stat().st_size > 100000:
        raise ValueError("oversized calibration")
    meta = json.loads(path.read_text())
    model, parent_meta = compare.load_model(parent, device)
    if (
        meta["format"] != "pour-relative-level-v1"
        or meta["parent_checkpoint_sha256"] != parent_meta["checkpoint_sha256"]
    ):
        raise ValueError("calibration/texture identity mismatch")
    anchor = controls.copy()
    anchor[4] = 0
    first = p.decode(p.sample(model, anchor, seed), 314)
    base = p.decode(p.sample(model, controls, seed), 314)
    wave, gain = calibrate(
        base, first, float(controls[4]), meta["phase_slopes_db"]["relative"]
    )
    if abs(wave).max() * audition_gain > 0.98:
        raise ValueError("requested audition gain exceeds headroom")
    output.mkdir(parents=True)
    path = output / "generated.wav"
    sf.write(path, wave * audition_gain, p.RATE, subtype="PCM_16")
    p.save_json(
        output / "result.json",
        {
            "reference_audio_input": False,
            "controls": controls.tolist(),
            "seed": seed,
            "decoder_seed": 314,
            "parent_checkpoint_sha256": parent_meta["checkpoint_sha256"],
            "calibration_sha256": hashlib.sha256(
                (calibration / "model.json").read_bytes()
            ).hexdigest(),
            "calibration_gain": gain,
            "audition_gain": audition_gain,
            "wav": str(path),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        },
    )


def run(source, parent, output):
    source, parent, output = (x.resolve() for x in (source, parent, output))
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("fresh external output required")
    torch.set_num_threads(4)
    rows, provenance = p.load_source(source)
    model, parent_meta = compare.load_model(parent, "cuda")
    training = [r for r in rows if r["role"] == "train"]
    if (
        parent_meta["train_ids"] != [r["item_id"] for r in training]
        or parent_meta["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    ):
        raise ValueError("source/training exposure mismatch")
    pairs = [
        [compare.phase_crop(row, phase) for phase in ("first", "middle")]
        for row in training
    ]
    controls = np.array([[x[1] for x in pair] for pair in pairs])
    levels = np.array([[level(x[2]) for x in pair] for pair in pairs])
    weights = coefficients(controls, levels)
    meta = {
        "format": "pour-relative-level-v1",
        "parent_checkpoint_sha256": parent_meta["checkpoint_sha256"],
        "train_ids": parent_meta["train_ids"],
        "source_sha256": parent_meta["source_sha256"],
        "phase_slopes_db": {k: float(v[5]) for k, v in weights.items()},
        "weights": {k: v.tolist() for k, v in weights.items()},
        "ridge_penalty": 0.01,
        "shuffle_seed": 53,
        "train_mean_phase_delta_db": float(np.mean(levels[:, 1] - levels[:, 0])),
        "scope": "only phase-relative level, anchored by generated first-window RMS; no absolute SPL, material or geometry calibration",
    }
    output.mkdir(parents=True)
    p.save_json(output / "model.json", meta)
    report = {
        "status": "complete",
        "scope": "all disclosed30 recordings/two objects; three seeds; not an independent test",
        "model": meta,
        "parent": parent_meta,
        "terms": provenance["terms"],
        "reference_audio_input_to_generator": False,
        "scalar_rows": [],
        "rows": [],
    }
    tags = {
        "status": "complete",
        "seconds": p.SAMPLES / p.RATE,
        "cases": [],
        "rows": [],
        "controls": [],
    }
    preview = []
    for idx, row in enumerate(r for r in rows if r["role"] == "unseen_container"):
        patches = [compare.phase_crop(row, q) for q in ("first", "middle")]
        c = np.stack([x[1] for x in patches])
        y = np.array([level(x[2]) for x in patches])
        dx = c[1] - c[0]
        predictions = {k: float(np.r_[0, dx] @ v) for k, v in weights.items()}
        predictions["train-mean-delta"] = meta["train_mean_phase_delta_db"]
        report["scalar_rows"].append(
            {
                "item_id": row["item_id"],
                "container_id": row["container_id"],
                "real_delta_db": float(y[1] - y[0]),
                "predicted_delta_db": predictions,
            }
        )
        anchors = {
            seed: p.decode(p.sample(model, c[0], seed), 314)
            for seed in (314, 2718, 1618)
        }
        for phase_index, phase in enumerate(("first", "middle")):
            _, condition, real, offset = patches[phase_index]
            pending = [("real", None, real, 1.0)]
            for seed, anchor in anchors.items():
                base = (
                    anchor
                    if phase_index == 0
                    else p.decode(p.sample(model, condition, seed), 314)
                )
                pending.append(("base", seed, base, 1.0))
                for kind in ("relative", "absolute", "shuffled", "zero-phase"):
                    wave, gain = calibrate(
                        base, anchor, float(condition[4]), meta["phase_slopes_db"][kind]
                    )
                    pending.append((kind, seed, wave, gain))
            for kind, seed, wave, gain in pending:
                key = f"{idx:02d}-{phase}-{kind}-{seed}"
                path = output / (key + ".wav")
                sf.write(path, wave, p.RATE, subtype="PCM_16")
                record = {
                    "item_id": row["item_id"],
                    "container_id": row["container_id"],
                    "phase": phase,
                    "kind": kind,
                    "seed": seed,
                    "decoder_seed": 314,
                    "controls": condition.tolist(),
                    "start_sample": offset,
                    "wav": str(path),
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                    "calibration_gain": gain,
                    "audition_gain": 1,
                    **p.metrics(wave, real),
                }
                report["rows"].append(record)
                tags["cases"].append({"id": key, "diagnostic_id": "water-pour"})
                tags["rows"].append({**record, "case": len(tags["cases"]) - 1})
                if idx < 2 and (
                    kind == "real" or (seed == 314 and kind in ("base", "relative"))
                ):
                    preview.extend([wave, np.zeros(p.RATE // 2)])
        print({"evaluated": idx + 1, "total": 30}, flush=True)
    path = output / "comparison.wav"
    sf.write(path, np.concatenate(preview), p.RATE, subtype="PCM_16")
    report["comparison"] = {
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "gain": 1,
        "order": "first glass then PET; first/middle; real/base/relative, seed314; fixed decoder314",
    }
    p.save_json(output / "result.json", report)
    p.save_json(output / "tag-input.json", tags)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "parent", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    run(args.source, args.parent, args.output)
