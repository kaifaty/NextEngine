"""Training-only resonance diagnostic with known-signal and time-shuffle controls.

A smooth spectral ridge is not necessarily a fundamental or a physical mode.
Extracted components depend on source audio and are diagnostic reconstructions,
not reference-free generated sound or automatically accepted training labels.
"""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path

import numpy as np
import physical_sound_pouring_compare as compare
import physical_sound_pouring_pilot as p
import soundfile as sf
from scipy.ndimage import median_filter
from scipy.signal import istft, stft

FFT = 2048
HOP = 256
GRID = 250 * 2 ** (np.arange(441) / 96)
JUMP = 12
PENALTY = 0.5


def analyze(wave):
    if (
        wave.ndim != 1
        or not FFT <= len(wave) <= p.RATE * 60
        or not np.isfinite(wave).all()
        or np.max(abs(wave)) <= 0
    ):
        raise ValueError("need finite nonzero mono audio, 0.128–60 seconds")
    frequencies, times, spectrum = stft(
        wave, fs=p.RATE, nperseg=FFT, noverlap=FFT - HOP
    )
    power = abs(spectrum) ** 2
    db = 10 * np.log10(np.maximum(power / power.max(), 1e-12))
    whitened = db - median_filter(db, size=(31, 1), mode="nearest")
    emissions = np.stack(
        [np.interp(GRID, frequencies, col) for col in whitened.T], axis=1
    )
    return times, spectrum, np.clip(emissions, -20, 30)


def track(emissions):
    if (
        emissions.ndim != 2
        or emissions.shape[0] != len(GRID)
        or not np.isfinite(emissions).all()
    ):
        raise ValueError("invalid ridge scores")
    count, frames = emissions.shape
    if frames < 2:
        raise ValueError("need multiple time frames")
    states = np.arange(count)
    shifts = np.arange(-JUMP, JUMP + 1)
    predecessors = states[None] + shifts[:, None]
    valid = (predecessors >= 0) & (predecessors < count)
    predecessors = predecessors.clip(0, count - 1)
    costs = PENALTY * abs(shifts[:, None])
    score = emissions[:, 0].copy()
    back = np.zeros((frames, count), dtype=np.int16)
    for frame in range(1, frames):
        options = np.where(valid, score[predecessors] - costs, -np.inf)
        choice = options.argmax(0)
        back[frame] = predecessors[choice, states]
        score = options[choice, states] + emissions[:, frame]
    path = np.zeros(frames, dtype=np.int16)
    path[-1] = score.argmax()
    total = float(score[path[-1]] / frames)
    for frame in range(frames - 1, 0, -1):
        path[frame - 1] = back[frame, path[frame]]
    return path, total


def reconstruct(wave, times, spectrum, path):
    if len(path) != len(times) or spectrum.shape[1] != len(path):
        raise ValueError("unaligned ridge")
    hz = GRID[path]
    frequencies = np.arange(FFT // 2 + 1) * p.RATE / FFT
    # Gaussian band with 80-cent standard deviation; zero DC.
    cents = 1200 * np.log2(np.maximum(frequencies[:, None], 1) / hz[None])
    mask = np.exp(-0.5 * (cents / 80) ** 2)
    mask[0] = 0
    ridge = istft(spectrum * mask, fs=p.RATE, nperseg=FFT, noverlap=FFT - HOP)[1][
        : len(wave)
    ]
    residual = wave - ridge
    if (
        not np.isfinite(ridge).all()
        or max(abs(ridge).max(), abs(residual).max()) > 0.98
    ):
        raise ValueError("component exceeds headroom")
    return ridge.astype(np.float32), residual.astype(np.float32)


def synthetic(kind, seed=53):
    if kind not in ("rising", "falling", "stationary", "crossing", "noise"):
        raise ValueError("unknown synthetic control")
    times = np.arange(p.SAMPLES) / p.RATE
    position = times / times[-1]
    rising = 600 * (2500 / 600) ** position
    falling = 2400 * (650 / 2400) ** position
    hz = {
        "rising": rising,
        "falling": falling,
        "stationary": np.full(len(times), 1000),
        "crossing": rising,
        "noise": rising,
    }[kind]
    phase = 2 * np.pi * np.cumsum(hz) / p.RATE
    wave = np.random.default_rng(seed).normal(0, 0.012, len(times))
    targets = []
    if kind != "noise":
        wave += 0.03 * np.sin(phase) + 0.009 * np.sin(3 * phase)
        targets.append(hz)
    if kind == "crossing":
        wave += 0.03 * np.sin(2 * np.pi * np.cumsum(falling) / p.RATE)
        targets.append(falling)
    return wave.astype(np.float32), times, targets


def inspect(wave):
    times, spectrum, emissions = analyze(wave)
    path, score = track(emissions)
    rng = np.random.default_rng(53)
    null_scores = [
        track(emissions[:, rng.permutation(len(times))])[1] for _ in range(9)
    ]
    reverse, reverse_score = track(emissions[:, ::-1])
    ridge, residual = reconstruct(wave, times, spectrum, path)
    report = {
        "time_seconds": times.tolist(),
        "ridge_hz": GRID[path].tolist(),
        "path_score": score,
        "shuffled_scores": null_scores,
        "score_above_max_shuffle": score - max(null_scores),
        "reversal_score_error": abs(score - reverse_score),
        "reversal_median_cents": float(
            np.median(abs(1200 * np.log2(GRID[path] / GRID[reverse[::-1]])))
        ),
        "median_prominence_db": float(
            np.median(emissions[path, np.arange(len(times))])
        ),
        "ridge_energy_fraction": float(np.mean(ridge**2) / np.mean(wave**2)),
        "reconstruction_max_error": float(abs(ridge + residual - wave).max()),
        "automatic_label_admission": False,
    }
    return report, ridge, residual


def run(source, output):
    source, output = source.resolve(), output.resolve()
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("fresh external output required")
    rows, provenance = p.load_source(source)
    training = compare.select_rows(rows, True)
    assert len(training) == 13 and all(r["role"] == "train" for r in training)
    output.mkdir(parents=True)
    report = {
        "status": "running",
        "scope": "first source-order recording per13 training objects only; no new physical labels",
        "source_sha256": hashlib.sha256(
            (source / "source.json").read_bytes()
        ).hexdigest(),
        "terms": provenance["terms"],
        "fft": FFT,
        "hop": HOP,
        "bins_per_octave": 96,
        "max_jump_bins": JUMP,
        "penalty": PENALTY,
        "synthetic": [],
        "training": [],
        "wavs": [],
    }
    preview = []

    def save(name, wave):
        if not np.isfinite(wave).all() or abs(wave).max() > 0.98:
            raise ValueError("unsafe audition; no silent normalization")
        path = output / (name + ".wav")
        sf.write(path, wave, p.RATE, subtype="PCM_16")
        report["wavs"].append(
            {
                "wav": str(path),
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "gain": 1,
            }
        )

    for kind in ("rising", "falling", "stationary", "crossing", "noise"):
        wave, times, targets = synthetic(kind)
        result, ridge, residual = inspect(wave)
        if targets:
            predictions = np.asarray(result["ridge_hz"])
            truth = np.stack(
                [np.interp(result["time_seconds"], times, hz) for hz in targets]
            )
            error = np.min(abs(1200 * np.log2(predictions[None] / truth)), axis=0)[4:-4]
            result.update(
                {
                    "median_error_cents": float(np.median(error)),
                    "within50cents_fraction": float(np.mean(error < 50)),
                    "target_scope": "nearest active mode; crossing does not prove mode identity",
                }
            )
        report["synthetic"].append({"kind": kind, **result})
        for label, sound in (("input", wave), ("ridge", ridge), ("residual", residual)):
            save(f"synthetic-{kind}-{label}", sound)
    for idx, row in enumerate(training):
        result, ridge, residual = inspect(row["wave"])
        report["training"].append(
            {
                "item_id": row["item_id"],
                "container_id": row["container_id"],
                "material": row["material"],
                "shape": row["shape"],
                **result,
            }
        )
        for label, sound in (
            ("real", row["wave"]),
            ("ridge", ridge),
            ("residual", residual),
        ):
            save(f"{idx:02d}-{row['container_id']}-{label}", sound)
            if idx < 2:
                preview.extend([sound, np.zeros(p.RATE // 2)])
        print({"training_objects": idx + 1, "total": len(training)}, flush=True)
    save("comparison", np.concatenate(preview))
    report["comparison_order"] = (
        "first two source-order training objects; full real/ridge/residual; gain1; source-dependent diagnostic, not neural generation"
    )
    report["status"] = "complete"
    p.save_json(output / "result.json", report)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    run(args.source, args.output)
