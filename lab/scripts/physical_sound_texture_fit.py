"""Small physical-condition -> stochastic friction sound baseline, offline only.

Known surfaces and fixed rubber probe; no claim of universal material synthesis.
Train repeat0 at20/30/50/60mm/s;40mm/s and repeat1 are disclosed development.
The network predicts a stationary spectrum, not recorded phase or impacts.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import physical_sound_texture_probe as source
import soundfile as sf
import torch
from safetensors.torch import load_file, save_file
from scipy.signal import resample_poly, welch
from torch import nn

RATE = 22050
FFT = 1024
TEXTURES = tuple(source.TEXTURES)


def features(texture: int, speed: float, force: float) -> np.ndarray:
    if (
        texture not in TEXTURES
        or not np.isfinite([speed, force]).all()
        or not 20 <= speed <= 60
        or not 0.5 <= force <= 1
    ):
        raise ValueError("outside known texture/speed/force domain")
    return np.array(
        [float(texture == item) for item in TEXTURES]
        + [(speed - 40) / 20, (force - 0.75) / 0.25],
        dtype=np.float32,
    )


def role(row: dict) -> str:
    if row["commanded_speed_mm_s"] == 40:
        return "unseen_speed"
    return "train" if row["repeat"] == 0 else "repeat_development"


class SpectrumNet(nn.Module):
    def __init__(self, rank: int = 0):
        super().__init__()
        if rank not in (0, 4):
            raise ValueError("only full spectrum or four-component control supported")
        self.rank = rank
        self.layers = nn.Sequential(
            nn.Linear(5, 64),
            nn.SiLU(),
            nn.Linear(64, 64),
            nn.SiLU(),
            nn.Linear(64, rank or 513),
        )
        self.register_buffer("mean", torch.zeros(513))
        if rank:
            self.register_buffer("basis", torch.zeros(rank, 513))
        nn.init.zeros_(self.layers[-1].weight)
        nn.init.zeros_(self.layers[-1].bias)

    def forward(self, x):
        output = 10 * self.layers(x)
        return self.mean + (output @ self.basis if self.rank else output)


def spectrum(wave: np.ndarray) -> np.ndarray:
    if wave.ndim != 1 or len(wave) < FFT or not np.isfinite(wave).all():
        raise ValueError("invalid waveform")
    _, power = welch(wave, fs=RATE, nperseg=FFT, noverlap=FFT // 2)
    return (10 * np.log10(np.maximum(power, 1e-18))).astype(np.float32)


def synthesize(db: np.ndarray, seconds: float, seed: int) -> np.ndarray:
    if (
        db.shape != (513,)
        or not np.isfinite(db).all()
        or np.min(db) < -180
        or np.max(db) > 0
        or not np.isfinite(seconds)
        or not 0.25 <= seconds <= 10
        or not 0 <= seed < 2**32
    ):
        raise ValueError("invalid synthesis request")
    n = round(seconds * RATE)
    freq = np.fft.rfftfreq(n, 1 / RATE)
    power = 10 ** (np.interp(freq, np.fft.rfftfreq(FFT, 1 / RATE), db) / 10)
    transfer = np.sqrt(power * RATE / 2)
    transfer[0] = 0  # No DC component.
    if n % 2 == 0:
        transfer[-1] *= np.sqrt(2)
    wave = np.fft.irfft(
        np.fft.rfft(np.random.default_rng(seed).normal(size=n)) * transfer, n
    )
    fade = np.linspace(0, 1, round(0.01 * RATE))
    wave[: len(fade)] *= fade
    wave[-len(fade) :] *= fade[::-1]
    return wave.astype(np.float32)


def checked(path: str, manifest: dict, root: Path) -> Path:
    p = Path(path).resolve()
    if not p.is_relative_to(root) or not 0 < p.stat().st_size <= 4_000_000:
        raise ValueError("invalid source path or size")
    entry = next(item for item in manifest["files"] if item["path"] == str(p))
    if hashlib.sha256(p.read_bytes()).hexdigest() != entry["sha256"]:
        raise ValueError("source hash mismatch")
    return p


def load_data(path: Path):
    if path.stat().st_size > 2_000_000:
        raise ValueError("oversized source manifest")
    manifest = json.loads(path.read_text())
    if (
        manifest["status"] != "complete"
        or manifest["article"] != source.ARTICLE
        or not manifest.get("training_grid")
        or len(manifest["rows"]) != 60
        or manifest["license"]["name"] != "CC BY 4.0"
    ):
        raise ValueError("complete fixed source grid required")
    expected = {row["id"]: row for row in source.conditions(True)}
    seen = set()
    rows, waves, noises, inputs = [], [], [], []
    for original in manifest["rows"]:
        ident = original["id"]
        if ident not in expected or ident in seen:
            raise ValueError("duplicate or unexpected condition")
        seen.add(ident)
        row = expected[ident]
        if any(original[key] != value for key, value in row.items()):
            raise ValueError("condition metadata mismatch")
        position = np.genfromtxt(
            checked(original["position"], manifest, path.parent),
            delimiter=",",
            names=True,
        )
        force = np.genfromtxt(
            checked(original["force"], manifest, path.parent), delimiter=",", names=True
        )
        for record in (position, force):
            if not all(
                np.isfinite(record[name]).all() for name in record.dtype.names
            ) or not np.all(np.diff(record["time"]) > 0):
                raise ValueError("invalid sensor samples")
        middle = np.abs(position["Y"]) <= 27
        times = position["time"][middle]
        if len(times) < 10 or times[-1] - times[0] < 0.75:
            raise ValueError("insufficient central sliding span")
        start = (times[0] + times[-1]) / 2 - 0.375
        signals = []
        for kind in ("audio", "raw_audio"):
            p = checked(original[kind], manifest, path.parent)
            source.validate_audio(p.read_bytes(), 1 if kind == "audio" else 2)
            wave, rate = sf.read(p, always_2d=True)
            crop = wave[round(start * rate) : round(start * rate) + round(0.75 * rate)]
            if len(crop) != round(0.75 * rate):
                raise ValueError("incomplete source crop")
            signals.append(resample_poly(crop[:, 0 if kind == "audio" else 1], 1, 2))
        ff = force["force"][(force["time"] >= start) & (force["time"] <= start + 0.75)]
        if not ff.size:
            raise ValueError("missing matching force")
        rows.append(
            {
                **row,
                "role": role(row),
                "crop_start_seconds": float(start),
                "measured_force_median_N": float(np.median(ff)),
                "measured_speed_mm_s": float(
                    abs(np.polyfit(times, position["Y"][middle], 1)[0])
                ),
            }
        )
        waves.append(signals[0])
        noises.append(signals[1])
        inputs.append(
            features(
                row["texture_id"],
                row["commanded_speed_mm_s"],
                row["commanded_normal_force_N"],
            )
        )
    return rows, waves, noises, np.stack(inputs), manifest


def interpolate(row, train_rows, train_spectra):
    selected = sorted(
        (
            i
            for i, r in enumerate(train_rows)
            if r["texture_id"] == row["texture_id"]
            and r["commanded_normal_force_N"] == row["commanded_normal_force_N"]
        ),
        key=lambda i: train_rows[i]["commanded_speed_mm_s"],
    )
    speeds = [train_rows[i]["commanded_speed_mm_s"] for i in selected]
    return np.array(
        [
            np.interp(row["commanded_speed_mm_s"], speeds, train_spectra[selected, j])
            for j in range(513)
        ],
        dtype=np.float32,
    )


def publish(path: Path, wave: np.ndarray, gain: float) -> dict:
    if not np.isfinite(wave).all() or np.max(np.abs(wave * gain)) > 0.981:
        raise ValueError("invalid or overflowing output")
    sf.write(path, wave * gain, RATE, subtype="PCM_16")
    return {
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "playback_gain": gain,
    }


def waveform_metrics(candidate: np.ndarray, reference: np.ndarray) -> dict:
    def envelope_cv(wave):
        hop = 551  # Approximately25ms at22.05kHz.
        blocks = wave[: len(wave) // hop * hop].reshape(-1, hop)
        envelope = np.sqrt(np.mean(blocks**2, axis=1))
        return float(np.std(envelope) / max(np.mean(envelope), 1e-12))

    delta = spectrum(candidate)[1:] - spectrum(reference)[1:]
    return {
        "waveform_spectrum_rmse_db": float(np.sqrt(np.mean(delta**2))),
        "envelope_cv": envelope_cv(candidate),
        "real_envelope_cv": envelope_cv(reference),
        "level_error_db": float(
            10
            * np.log10(
                max(np.mean(candidate**2), 1e-18) / max(np.mean(reference**2), 1e-18)
            )
        ),
    }


def fit(manifest: Path, output: Path, rank: int = 0):
    torch.set_num_threads(2)
    torch.manual_seed(23)
    rows, waves, noises, inputs, corpus = load_data(manifest.resolve())
    output.mkdir(parents=True, exist_ok=False)
    train = np.array([row["role"] == "train" for row in rows])
    target = np.stack([spectrum(w) for w in waves])
    train_rows = [row for row in rows if row["role"] == "train"]
    model = SpectrumNet(rank)
    model.mean.copy_(torch.from_numpy(target[train].mean(axis=0)))
    x, y = torch.from_numpy(inputs[train]), torch.from_numpy(target[train])
    if rank:
        model.basis.copy_(
            torch.linalg.svd(y - model.mean, full_matrices=False).Vh[:rank]
        )
    optimizer = torch.optim.AdamW(model.parameters(), lr=1e-3, weight_decay=1e-4)
    for step in range(1500):
        loss = ((model(x) - y) / 10).square().mean()
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
    model.eval()
    save_file(model.state_dict(), output / "model.safetensors")
    meta = {
        "format": "texture-spectrum-v1",
        "rank": rank,
        "checkpoint_sha256": hashlib.sha256(
            (output / "model.safetensors").read_bytes()
        ).hexdigest(),
        "training_steps": 1500,
        "seed": 23,
        "corpus_sha256": hashlib.sha256(manifest.read_bytes()).hexdigest(),
        "license": corpus["license"],
        "authors": corpus["authors"],
        "input": "known surface ID, commanded speed mm/s, commanded force N; fixed urethane probe",
        "output": "stationary Gaussian texture; not a complete physical object model",
    }
    (output / "model.json").write_text(json.dumps(meta, indent=2) + "\n")
    with torch.no_grad():
        pred = model(torch.from_numpy(inputs)).numpy()
    interpolated = np.stack(
        [interpolate(row, train_rows, target[train]) for row in rows]
    )
    # A single train-derived playback gain, shared by real/neural/baseline/oracle.
    gain = min(
        100,
        0.8
        / max(float(np.abs(w).max()) for w, t in zip(waves, train, strict=True) if t),
    )
    report = {
        "status": "complete",
        "scope": "disclosed known-surface interpolation, not independent naturalness or engine admission",
        "model": meta,
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "playback_gain": gain,
        "train_loss_scaled": float(loss.detach()),
        "rows": [],
        "clips": [],
    }
    comparisons = []
    for i, row in enumerate(rows):
        noise = spectrum(noises[i])
        noise += target[i].mean() - noise.mean()  # Shape control only; not SNR.
        error = lambda a, expected=target[i, 1:]: float(
            np.sqrt(np.mean((a[1:] - expected) ** 2))
        )
        report["rows"].append(
            {
                **row,
                "neural_spectrum_rmse_db": error(pred[i]),
                "interpolation_spectrum_rmse_db": error(interpolated[i]),
                "machine_shape_rmse_db": error(noise),
            }
        )
        if row["role"] == "unseen_speed" and row["repeat"] == 0:
            for label, wave in [
                ("real", waves[i]),
                ("neural", synthesize(pred[i], 0.75, 314)),
                ("interpolation", synthesize(interpolated[i], 0.75, 314)),
                ("oracle-spectrum", synthesize(target[i], 0.75, 314)),
            ]:
                record = publish(output / f"{row['id']}-{label}.wav", wave, gain)
                report["clips"].append(
                    {
                        "id": row["id"],
                        "kind": label,
                        **record,
                        "waveform_metrics": waveform_metrics(wave, waves[i]),
                    }
                )
                comparisons.extend([wave, np.zeros(RATE // 4)])
    report["comparison"] = publish(
        output / "comparison.wav", np.concatenate(comparisons), gain
    )
    (output / "result.json").write_text(
        json.dumps(report, indent=2, allow_nan=False) + "\n"
    )
    print(json.dumps({"status": "complete", "comparison": report["comparison"]}))


def render(
    directory: Path,
    texture: int,
    speed: float,
    force: float,
    seconds: float,
    seed: int,
    output: Path,
):
    # This path deliberately never reads the dataset or any reference recording.
    metadata = directory / "model.json"
    checkpoint = directory / "model.safetensors"
    if metadata.stat().st_size > 65536 or checkpoint.stat().st_size > 1_000_000:
        raise ValueError("oversized model")
    meta = json.loads(metadata.read_text())
    if (
        meta["format"] != "texture-spectrum-v1"
        or hashlib.sha256(checkpoint.read_bytes()).hexdigest()
        != meta["checkpoint_sha256"]
    ):
        raise ValueError("model identity mismatch")
    model = SpectrumNet(meta.get("rank", 0))
    state = load_file(checkpoint)
    if not all(torch.isfinite(value).all() for value in state.values()):
        raise ValueError("nonfinite weights")
    model.load_state_dict(state, strict=True)
    with torch.no_grad():
        db = model(torch.from_numpy(features(texture, speed, force))).numpy()
    wave = synthesize(db, seconds, seed)
    output.mkdir(parents=True, exist_ok=False)
    result = publish(
        output / "generated.wav",
        wave,
        min(100, 0.8 / max(float(np.abs(wave).max()), 1e-12)),
    )
    (output / "result.json").write_text(
        json.dumps(
            {
                "reference_audio_input": False,
                "texture": texture,
                "speed": speed,
                "force": force,
                "seed": seed,
                "model": meta,
                **result,
            },
            indent=2,
        )
        + "\n"
    )
    print(json.dumps(result))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--corpus", type=Path)
    group.add_argument("--render-model", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--texture", type=int, default=74)
    parser.add_argument("--speed", type=float, default=40)
    parser.add_argument("--force", type=float, default=0.5)
    parser.add_argument("--seconds", type=float, default=2)
    parser.add_argument("--seed", type=int, default=314)
    parser.add_argument("--rank", type=int, choices=(0, 4), default=0)
    args = parser.parse_args()
    out = args.output.resolve()
    if out.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("generated artifacts must stay outside the repository")
    if args.corpus:
        fit(args.corpus, out, args.rank)
    else:
        render(
            args.render_model,
            args.texture,
            args.speed,
            args.force,
            args.seconds,
            args.seed,
            out,
        )
