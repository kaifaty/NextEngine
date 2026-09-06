"""Rain accumulation -> neural stationary forest soundscape, report-only.

DataSuds10.23708/I0QYNM V2, Xavier et al., CC-BY4.0. Source labels are
accumulated mm/5min, not instantaneous intensity. No cross-site tables used.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import pandas as pd
import soundfile as sf
import torch
from physical_sound_texture_fit import synthesize
from safetensors.torch import load_file, save_file
from scipy.signal import welch
from torch import nn

CSV_BYTES = 381270141
CSV_MD5 = "d91a06cecf3af48a205bdf43c48abee1"
RATE = 48000
EXAMPLES = (
    "SMM00894_20230510_223500.wav",
    "SMM00894_20230510_224500.wav",
    "SMM00894_20230510_225500.wav",
)


def split_days(days: np.ndarray) -> np.ndarray:
    unique = sorted(set(days.tolist()))
    development = {
        day
        for day in unique
        if int(hashlib.sha256(str(day).encode()).hexdigest()[:8], 16) % 5 == 0
    } | {np.datetime64("2023-05-10", "D").astype(int)}
    # Guard adjacent days; this does not prove independence of multi-day storms.
    guard = {d + offset for d in development for offset in (-1, 1)} - development
    return np.array(
        [
            "development" if d in development else "guard" if d in guard else "train"
            for d in days
        ]
    )


def digital_psd(power: np.ndarray) -> np.ndarray:
    if not np.isfinite(power).all() or np.min(power) <= 0:
        raise ValueError("expected positive finite PCM16-domain PSD")
    return (10 * np.log10(power / 32768**2)).astype(np.float32)


class RainNet(nn.Module):
    def __init__(self):
        super().__init__()
        self.layers = nn.Sequential(
            nn.Linear(1, 32), nn.SiLU(), nn.Linear(32, 32), nn.SiLU(), nn.Linear(32, 8)
        )
        self.register_buffer("mean", torch.zeros(513))
        self.register_buffer("basis", torch.zeros(8, 513))
        self.register_buffer("scale", torch.ones(8))
        nn.init.zeros_(self.layers[-1].weight)
        nn.init.zeros_(self.layers[-1].bias)

    def forward(self, rain):
        return self.mean + (self.layers(torch.log1p(rain)) * self.scale) @ self.basis


def save_json(path: Path, data):
    path.write_text(json.dumps(data, indent=2, allow_nan=False) + "\n")


def publish(path: Path, wave: np.ndarray, gain: float):
    if not np.isfinite(wave).all() or abs(wave * gain).max() > 0.981:
        raise ValueError("invalid output")
    sf.write(path, wave * gain, RATE, subtype="PCM_16")
    return {
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "gain": gain,
        "raw_rms": float(np.sqrt(np.mean(wave**2))),
    }


def load_corpus(source: Path):
    csv = source / "psds_training.csv"
    if csv.stat().st_size != CSV_BYTES:
        raise ValueError("wrong CSV size; obtain format=original, not converted TSV")
    with csv.open("rb") as stream:
        if hashlib.file_digest(stream, "md5").hexdigest() != CSV_MD5:
            raise ValueError("training source checksum mismatch")
    frame = pd.read_csv(csv)
    expected = np.rint(np.arange(513) * RATE / 1024).astype(int).astype(str).tolist()
    if (
        frame.shape != (48208, 516)
        or list(frame.columns) != ["timestamp", "total_rain", "filename"] + expected
    ):
        raise ValueError("unexpected spectral table")
    dates = pd.to_datetime(frame.timestamp, errors="raise")
    if (
        not frame.filename.str.fullmatch(r"SMM00894_[0-9]{8}_[0-9]{6}\.wav").all()
        or frame.filename.duplicated().any()
    ):
        raise ValueError("source identity/timestamp mismatch")
    starts = pd.to_datetime(frame.filename.str[9:-4], format="%Y%m%d_%H%M%S")
    # 33 recordings start seconds after their minute-aligned table timestamp.
    # Retain true filenames; this is not sample-level rain-gauge alignment.
    if not (starts.dt.floor("min") == dates).all():
        raise ValueError("recording does not belong to the labelled minute")
    rain = frame.total_rain.to_numpy(dtype=np.float32)
    if not np.isfinite(rain).all() or np.min(rain) < 0 or np.max(rain) > 100:
        raise ValueError("invalid measured rain")
    target = digital_psd(frame.iloc[:, 3:].to_numpy(dtype=float))
    days = dates.to_numpy().astype("datetime64[D]").astype(int)
    roles = split_days(days)
    metadata = json.loads((source / "metadata.json").read_text())["data"]
    controls = []
    for name in EXAMPLES:
        record = next(
            x["dataFile"]
            for x in metadata["files"]
            if x["dataFile"]["filename"] == name
        )
        path = source / name
        if (
            path.stat().st_size != record["filesize"]
            or hashlib.md5(path.read_bytes()).hexdigest() != record["checksum"]["value"]
        ):
            raise ValueError("example checksum mismatch")
        wave, rate = sf.read(path)
        if wave.ndim != 1 or rate != RATE or len(wave) != RATE * 60:
            raise ValueError("example format mismatch")
        index = int(np.flatnonzero(frame.filename.to_numpy() == name)[0])
        _, psd = welch(wave * 32768, fs=RATE, nperseg=1024)
        error = 10 * np.log10(psd / frame.iloc[index, 3:].to_numpy(dtype=float))
        if abs(error).max() > 0.001:
            raise ValueError("source PSD units or estimator mismatch")
        controls.append(
            {
                "name": name,
                "index": index,
                "rain_mm_5min": float(rain[index]),
                "psd_max_error_db": float(abs(error).max()),
                "wave": wave,
            }
        )
    return frame, rain, target, days, roles, controls, metadata


def run(source: Path, output: Path):
    torch.set_num_threads(2)
    torch.manual_seed(41)
    frame, rain, target, days, roles, controls, metadata = load_corpus(source)
    output.mkdir(parents=True, exist_ok=False)
    rng = np.random.default_rng(41)
    wet = np.flatnonzero((roles == "train") & (rain > 0))
    dry = np.flatnonzero((roles == "train") & (rain == 0))
    if not len(wet) or len(dry) < len(wet):
        raise ValueError("insufficient training coverage")
    # Equal dry/wet sampling; all wet records retained. Selection never sees development spectra.
    training = np.concatenate([wet, rng.choice(dry, len(wet), replace=False)])
    y = torch.from_numpy(target[training])
    x = torch.from_numpy(rain[training, None])
    model = RainNet()
    model.mean.copy_(y.mean(0))
    _, singular, basis = torch.linalg.svd(y - model.mean, full_matrices=False)
    model.basis.copy_(basis[:8])
    model.scale.copy_((singular[:8] / np.sqrt(len(y) - 1)).clamp_min(1))
    optimizer = torch.optim.AdamW(model.parameters(), lr=1e-3, weight_decay=1e-4)
    for step in range(2000):
        indices = torch.randint(len(y), (128,))
        loss = ((model(x[indices]) - y[indices]) / 10).square().mean()
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
    model.eval()
    with torch.no_grad():
        pred = model(torch.from_numpy(rain[:, None])).numpy()
    # Same selected training rows for the non-neural exact-accumulation mean/interpolator.
    levels = np.unique(rain[training])
    means = np.stack(
        [target[training[rain[training] == level]].mean(0) for level in levels]
    )
    baseline = np.stack(
        [np.interp(rain, levels, means[:, k]) for k in range(513)], axis=1
    )
    error = np.sqrt(np.mean((pred[:, 1:] - target[:, 1:]) ** 2, axis=1))
    base_error = np.sqrt(np.mean((baseline[:, 1:] - target[:, 1:]) ** 2, axis=1))
    save_file(model.state_dict(), output / "model.safetensors")
    meta = {
        "format": "rain-spectrum-v1",
        "accumulation_unit": "mm per five minutes",
        "maximum_mm_5min": float(rain[training].max()),
        "seed": 41,
        "steps": 2000,
        "rank": 8,
        "selected_train_rows": len(training),
        "source_csv_md5": CSV_MD5,
        "checkpoint_sha256": hashlib.sha256(
            (output / "model.safetensors").read_bytes()
        ).hexdigest(),
        "source": "doi:10.23708/I0QYNM V2",
        "terms": metadata["termsOfUse"],
        "scope": "one forest recorder; stationary soundscape, not isolated rain or instantaneous physical calibration",
    }
    save_json(output / "model.json", meta)
    report = {
        "status": "complete",
        "model": meta,
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "groups": [],
        "source_controls": [
            {k: v for k, v in c.items() if k != "wave"} for c in controls
        ],
        "clips": [],
    }
    for role in ("train", "development", "guard"):
        for wet_only in (False, True):
            mask = (roles == role) & (
                (rain > 0) if wet_only else np.ones(len(rain), dtype=bool)
            )
            if not mask.any():
                continue
            report["groups"].append(
                {
                    "role": role,
                    "wet_only": wet_only,
                    "rows": int(mask.sum()),
                    "days": len(np.unique(days[mask])),
                    "neural_rmse_db": float(error[mask].mean()),
                    "interpolation_rmse_db": float(base_error[mask].mean()),
                    "neural_wins": int((error[mask] < base_error[mask]).sum()),
                }
            )
    # Preserve exact source/day membership and results without committing dataset rows.
    selected = set(training.tolist())
    save_json(
        output / "rows.json",
        [
            {
                "filename": name,
                "role": str(role),
                "selected_train": int(i) in selected,
                "rain_mm_5min": float(rain[i]),
                "neural_rmse_db": float(error[i]),
                "interpolation_rmse_db": float(base_error[i]),
            }
            for i, (name, role) in enumerate(zip(frame.filename, roles, strict=True))
        ],
    )
    pending = []
    for control in controls:
        i = control["index"]
        for kind, wave in [
            ("real", control["wave"][: RATE * 8]),
            ("neural", synthesize(pred[i], 8, 314, RATE)),
            ("interpolation", synthesize(baseline[i], 8, 314, RATE)),
            ("oracle", synthesize(target[i], 8, 314, RATE)),
        ]:
            pending.append(
                (
                    f"{control['rain_mm_5min']:.1f}-{kind}",
                    wave,
                    control["rain_mm_5min"],
                    kind,
                )
            )
    gain = min(1, 0.98 / max(float(abs(w).max()) for _, w, _, _ in pending))
    preview = []
    for name, wave, amount, kind in pending:
        report["clips"].append(
            {
                "kind": kind,
                "rain_mm_5min": amount,
                **publish(output / (name + ".wav"), wave, gain),
            }
        )
        preview.extend([wave, np.zeros(RATE // 2)])
    report["comparison"] = publish(
        output / "comparison.wav", np.concatenate(preview), gain
    )
    save_json(output / "result.json", report)
    print(json.dumps(report["groups"]), flush=True)


def render(directory: Path, amount: float, seed: int, output: Path):
    checkpoint = directory / "model.safetensors"
    metadata = directory / "model.json"
    if checkpoint.stat().st_size > 100000 or metadata.stat().st_size > 65536:
        raise ValueError("oversized model")
    meta = json.loads(metadata.read_text())
    if (
        meta["format"] != "rain-spectrum-v1"
        or not np.isfinite(amount)
        or not 0 <= amount <= meta["maximum_mm_5min"]
        or hashlib.sha256(checkpoint.read_bytes()).hexdigest()
        != meta["checkpoint_sha256"]
    ):
        raise ValueError("model identity or request outside domain")
    model = RainNet()
    state = load_file(checkpoint)
    if not all(torch.isfinite(value).all() for value in state.values()):
        raise ValueError("nonfinite weights")
    model.load_state_dict(state, strict=True)
    with torch.no_grad():
        db = model(torch.tensor([[amount]], dtype=torch.float32)).numpy()[0]
    wave = synthesize(db, 8, seed, RATE)
    output.mkdir(parents=True, exist_ok=False)
    result = {
        "model": meta,
        "reference_audio_input": False,
        "rain_mm_5min": amount,
        "seed": seed,
        **publish(
            output / "generated.wav",
            wave,
            min(1, 0.98 / max(float(abs(wave).max()), 1e-12)),
        ),
    }
    save_json(output / "result.json", result)
    print(json.dumps({"wav": result["wav"]}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--source", type=Path)
    group.add_argument("--render-model", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--amount", type=float, default=2)
    parser.add_argument("--seed", type=int, default=2718)
    args = parser.parse_args()
    out = args.output.resolve()
    if out.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("keep data/models/media outside the repository")
    if args.source:
        run(args.source.resolve(), out)
    else:
        render(args.render_model.resolve(), args.amount, args.seed, out)
