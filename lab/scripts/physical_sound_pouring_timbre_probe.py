"""Test paired, gain-invariant timbral evolution before another neural fit.

Linear controls diagnose whether the available metadata predicts coarse spectral
change. Filtering uses generated anchor/current audio only, not real references.
This is a report-only learned correction, not calibrated material acoustics.
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
from scipy.signal import istft


def profile(wave):
    if wave.shape != (p.SAMPLES,) or not np.isfinite(wave).all():
        raise ValueError("invalid waveform")
    power = np.mean(abs(p.transform(wave)[1:]) ** 2, axis=1).reshape(32, 8).mean(1)
    if power.max() <= 0:
        raise ValueError("zero-energy waveform")
    db = 10 * np.log10(np.maximum(power / power.max(), 1e-10))
    return db - db.mean()


def features(controls):
    x = np.asarray(controls, dtype=np.float64).copy()
    if x.ndim != 2 or x.shape[1] != 11 or not np.isfinite(x).all():
        raise ValueError("invalid controls")
    progress = x[:, 4].copy()
    if (progress < 0).any() or (progress > 1).any():
        raise ValueError("invalid progress")
    x[:, 4] = 1
    return x * progress[:, None]


def fit(controls, targets):
    x = features(controls)
    targets = np.asarray(targets, dtype=np.float64)
    if targets.shape != (len(x), 32) or not np.isfinite(targets).all():
        raise ValueError("invalid targets")
    signs = np.random.default_rng(53).choice([-1, 1], size=(len(x), 1))

    def solve(a, y):
        return np.linalg.solve(a.T @ a + np.eye(a.shape[1]) * 0.01, a.T @ y)

    return {
        "conditioned": solve(x, targets),
        "global": solve(x[:, 4:5], targets),
        "shuffled": solve(x, targets * signs),
        "mean-delta": targets.mean(0),
    }


def predict(model, controls):
    x = features(controls)
    return {
        "conditioned": x @ model["conditioned"],
        "global": x[:, 4:5] @ model["global"],
        "shuffled": x @ model["shuffled"],
        "mean-delta": np.broadcast_to(model["mean-delta"], (len(x), 32)),
        "zero": np.zeros((len(x), 32)),
    }


def recolor(wave, anchor, delta):
    """Generated audio supplies both profiles; real recordings are not inputs."""
    delta = np.asarray(delta)
    if delta.shape != (32,) or not np.isfinite(delta).all():
        raise ValueError("invalid spectral correction")
    correction = profile(anchor) + delta - profile(wave)
    if abs(correction).max() > 60:
        raise ValueError("correction exceeds numerical guard")
    centers = np.arange(32) * 8 + 3.5
    db = np.interp(np.arange(256), centers, correction)
    spectrum = p.transform(wave)
    spectrum[1:] *= (10 ** (db / 20))[:, None]
    result = istft(spectrum, fs=p.RATE, nperseg=p.FFT, noverlap=p.FFT - p.HOP)[1]
    result = result[: p.SAMPLES]
    gain = np.sqrt(np.mean(wave**2) / max(np.mean(result**2), 1e-20))
    result = (result * gain).astype(np.float32)
    if not np.isfinite(result).all() or abs(result).max() > 0.98:
        raise ValueError("recoloring exceeds headroom; no automatic attenuation")
    return result


def run(source, base_outputs, output):
    source, base_outputs, output = (x.resolve() for x in (source, base_outputs, output))
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("fresh external output required")
    rows, provenance = p.load_source(source)
    base = json.loads((base_outputs / "result.json").read_text())
    training = [r for r in rows if r["role"] == "train"]
    if (
        base["parent"]["train_ids"] != [r["item_id"] for r in training]
        or base["parent"]["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    ):
        raise ValueError("source/exposure mismatch")
    patches = {
        r["item_id"]: [compare.phase_crop(r, phase) for phase in ("first", "middle")]
        for r in rows
    }

    def data(selected):
        pairs = [patches[r["item_id"]] for r in selected]
        return (
            np.stack([pair[1][1] for pair in pairs]),
            np.stack([profile(pair[1][2]) - profile(pair[0][2]) for pair in pairs]),
        )

    x, y = data(training)
    model = fit(x, y)
    report = {
        "status": "running",
        "scope": "disclosed development, two excluded objects; training-side group validation is not an independent neural test",
        "terms": provenance["terms"],
        "reference_audio_input_to_generator": False,
        "base_result_sha256": hashlib.sha256(
            (base_outputs / "result.json").read_bytes()
        ).hexdigest(),
        "group_validation": [],
        "development": [],
        "rows": [],
    }
    for group in sorted({r["container_id"] for r in training}):
        mask = np.array([r["container_id"] == group for r in training])
        predicted = predict(fit(x[~mask], y[~mask]), x[mask])
        report["group_validation"].append(
            {
                "container_id": group,
                "recordings": int(mask.sum()),
                "rmse_db": {
                    k: float(np.sqrt(np.mean((v - y[mask]) ** 2)))
                    for k, v in predicted.items()
                },
            }
        )
    output.mkdir(parents=True)
    p.save_json(
        output / "model.json",
        {
            "format": "pour-relative-timbre-probe-v1",
            "ridge_penalty": 0.01,
            "train_ids": [r["item_id"] for r in training],
            "source_sha256": base["parent"]["source_sha256"],
            "parent_checkpoint_sha256": base["parent"]["checkpoint_sha256"],
            "weights": {k: v.tolist() for k, v in model.items()},
            "scope": "32 linear-frequency bands, relative spectral color only; no physical/material accuracy claim",
        },
    )
    cache = {
        (r["item_id"], r["phase"], r["seed"]): r
        for r in base["rows"]
        if r["kind"] == "base"
    }

    def read_generated(item, phase, seed):
        record = cache[item, phase, seed]
        path = Path(record["wav"])
        if hashlib.sha256(path.read_bytes()).hexdigest() != record["sha256"]:
            raise ValueError("generated base integrity mismatch")
        wave, rate = sf.read(path, dtype="float32")
        if rate != p.RATE:
            raise ValueError("wrong generated rate")
        return wave

    preview = []
    for idx, row in enumerate(r for r in rows if r["role"] == "unseen_container"):
        item = row["item_id"]
        c, target = data([row])
        predicted = predict(model, c)
        scalar = {
            "item_id": item,
            "container_id": row["container_id"],
            "rmse_db": {
                k: float(np.sqrt(np.mean((v - target) ** 2)))
                for k, v in predicted.items()
            },
        }
        report["development"].append(scalar)
        real = patches[item][1][2]
        for seed in (314, 2718, 1618):
            anchor = read_generated(item, "first", seed)
            wave = read_generated(item, "middle", seed)
            pending = [("base", wave)] + [
                (kind, recolor(wave, anchor, predicted[kind][0]))
                for kind in ("global", "conditioned", "shuffled")
            ]
            for kind, sound in pending:
                path = output / f"{idx:02d}-{kind}-{seed}.wav"
                sf.write(path, sound, p.RATE, subtype="PCM_16")
                report["rows"].append(
                    {
                        "item_id": item,
                        "container_id": row["container_id"],
                        "kind": kind,
                        "seed": seed,
                        "phase": "middle",
                        "audition_gain": 1,
                        "wav": str(path),
                        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                        "relative_shape_rmse_db": float(
                            np.sqrt(
                                np.mean(
                                    (profile(sound) - profile(anchor) - target[0]) ** 2
                                )
                            )
                        ),
                        "shape_rmse_db": float(
                            np.sqrt(np.mean((profile(sound) - profile(real)) ** 2))
                        ),
                        **p.metrics(sound, real),
                    }
                )
            if idx < 2 and seed == 2718:
                for sound in [real] + [w for _, w in pending]:
                    preview.extend([sound, np.zeros(p.RATE // 2)])
    path = output / "comparison.wav"
    sf.write(path, np.concatenate(preview), p.RATE, subtype="PCM_16")
    report["comparison"] = {
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "order": "first source-order glass then PET, middle; real/base/global/conditioned/shuffled; seed2718; gain1",
    }
    report["status"] = "complete"
    p.save_json(output / "result.json", report)
    p.save_json(
        output / "tag-input.json",
        {
            "status": "complete",
            "seconds": p.SAMPLES / p.RATE,
            "cases": [
                {"id": Path(r["wav"]).stem, "diagnostic_id": "water-pour"}
                for r in report["rows"]
            ],
            "rows": [{**r, "case": i} for i, r in enumerate(report["rows"])],
            "controls": [],
        },
    )
    print(
        json.dumps({"status": "complete", "wav_count": len(report["rows"]) + 1}),
        flush=True,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "base-outputs", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    run(args.source, args.base_outputs, args.output)
