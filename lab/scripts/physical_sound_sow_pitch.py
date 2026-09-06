"""Frozen Sound of Water pitch diagnostic; never a naturalness/material judge.

Inference adapted from bpiyush/SoundOfWater at
2599de7f11d565ed78f48e4340938e0fc6ef6455 (audio_pitch/model.py).
Only the reviewed forward path is used, without executing downloaded Python,
Lightning, video loaders, eval-based configuration, or unrestricted pickle.

MIT License
Copyright (c) 2024 Piyush Bagad

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path

import numpy as np
import physical_sound_pouring_pilot as p
import physical_sound_pouring_resonance_probe as ridge
import soundfile as sf
import torch
from scipy.signal import istft, stft
from torch import nn
from transformers import Wav2Vec2Config, Wav2Vec2FeatureExtractor, Wav2Vec2Model

SHA256 = "2fa3d8cec1488ee65bb5a6e30f1b79716d8243bbe4ddc4c0687ce2a02c84303c"
FILENAME = "dsr9mf13_ep100_step12423_real_finetuned_with_cosupervision.pth"


def time_encoding(frames, duration, device, start_seconds=0):
    # Match upstream: inclusive time endpoints, floor at49Hz, width512, gain.01.
    ticks = (
        torch.linspace(start_seconds, start_seconds + duration, frames).to(device) * 49
    ).long()
    divisor = torch.exp(
        torch.arange(0, 512, 2, dtype=torch.float) * (-math.log(10000) / 512)
    ).to(device)
    phase = ticks[:, None].float() * divisor
    return torch.stack((phase.sin(), phase.cos()), dim=-1).reshape(frames, 512) * 0.01


class PitchModel(nn.Module):
    def __init__(self, config):
        super().__init__()
        self.net = Wav2Vec2Model(config)
        self.axial_head = nn.Linear(768, 64)
        self.radial_head = nn.Linear(768, 64)

    def forward(self, audio, start_seconds=0):
        features = self.net.feature_extractor(audio).transpose(1, 2)
        features = (
            features
            + time_encoding(
                features.shape[1], audio.shape[1] / p.RATE, audio.device, start_seconds
            )[None]
        )
        hidden, _ = self.net.feature_projection(features)
        hidden = self.net.encoder(
            hidden,
            attention_mask=None,
            output_attentions=False,
            output_hidden_states=False,
            return_dict=True,
        )[0]
        return self.axial_head(hidden).softmax(-1)


def load(directory, device="cuda"):
    path = directory / FILENAME
    if (
        path.stat().st_size != 377980520
        or hashlib.sha256(path.read_bytes()).hexdigest() != SHA256
    ):
        raise ValueError("wrong published checkpoint")
    meta = json.loads((directory / "source.json").read_text())
    for row in meta["files"]:
        item = Path(row["path"])
        if (
            not item.resolve().is_relative_to(directory.resolve())
            or hashlib.sha256(item.read_bytes()).hexdigest() != row["sha256"]
        ):
            raise ValueError("source/config identity mismatch")
    config = Wav2Vec2Config.from_json_file(directory / "backbone-config/config.json")
    if (config.hidden_size, config.num_hidden_layers, config.num_attention_heads) != (
        768,
        12,
        12,
    ) or config.conv_dim != [512] * 7:
        raise ValueError("unexpected backbone shape")
    config._attn_implementation = "eager"
    model = PitchModel(config)
    state = torch.load(path, map_location="cpu", weights_only=True)
    if len(state) != 215 or not all(
        isinstance(v, torch.Tensor) and torch.isfinite(v).all() for v in state.values()
    ):
        raise ValueError("invalid checkpoint tensors")
    state = {k.removeprefix("backbone."): v for k, v in state.items()}
    model.load_state_dict(state, strict=True)
    extractor = Wav2Vec2FeatureExtractor.from_pretrained(
        directory / "backbone-config", local_files_only=True
    )
    return model.to(device).eval(), extractor, meta


@torch.inference_mode()
def infer(model, extractor, wave, start_seconds=0):
    if (
        wave.ndim != 1
        or not 400 <= len(wave) <= p.RATE * 60
        or not np.isfinite(wave).all()
        or not np.isfinite(start_seconds)
        or not 0 <= start_seconds <= 60 - len(wave) / p.RATE
    ):
        raise ValueError("invalid16kHz mono waveform")
    audio = extractor(wave, sampling_rate=p.RATE, return_tensors="pt").input_values
    probabilities = (
        model(audio.to(next(model.parameters()).device), start_seconds)[0].cpu().numpy()
    )
    if not np.isfinite(probabilities).all() or not np.allclose(
        probabilities.sum(1), 1, atol=1e-5
    ):
        raise ValueError("invalid posterior")
    wavelength = probabilities @ np.linspace(0, 100, 64)
    frequencies = 34000 / np.maximum(wavelength, 1e-6)
    # Upstream visualization uses400-sample receptive field and320-sample hop.
    times = (np.arange(len(frequencies)) * 320 + 200) / p.RATE + start_seconds
    entropy = -(probabilities * np.log(np.maximum(probabilities, 1e-12))).sum(
        1
    ) / np.log(64)
    return {
        "time_seconds": times.tolist(),
        "axial_hz": frequencies.tolist(),
        "normalized_entropy": entropy.tolist(),
        "zero_bin_probability": probabilities[:, 0].tolist(),
        "median_max_probability": float(np.median(probabilities.max(1))),
        "automatic_label_admission": False,
    }, probabilities


def component(wave, times, hz):
    f, t, spectrum = stft(
        wave, fs=p.RATE, nperseg=ridge.FFT, noverlap=ridge.FFT - ridge.HOP
    )
    target = np.interp(t, times, hz)
    cents = 1200 * np.log2(np.maximum(f[:, None], 1) / target[None])
    mask = np.exp(-0.5 * (cents / 80) ** 2)
    mask[0] = 0
    result = istft(
        spectrum * mask, fs=p.RATE, nperseg=ridge.FFT, noverlap=ridge.FFT - ridge.HOP
    )[1][: len(wave)]
    if not np.isfinite(result).all() or abs(result).max() > 0.98:
        raise ValueError("unsafe component")
    return result.astype(np.float32)


def shuffle_blocks(wave):
    size = p.RATE // 4
    full = len(wave) // size * size
    order = np.random.default_rng(53).permutation(full // size)
    return np.r_[wave[:full].reshape(-1, size)[order].reshape(-1), wave[full:]]


def run(model_dir, probe, output):
    model_dir, probe, output = (x.resolve() for x in (model_dir, probe, output))
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("fresh external output required")
    torch.set_num_threads(4)
    model, extractor, provenance = load(model_dir)
    previous = json.loads((probe / "result.json").read_text())
    records = {Path(r["wav"]).name: r for r in previous["wavs"]}
    pending = []
    for idx, row in enumerate(previous["training"]):
        pending.append(
            (row["container_id"], f"{idx:02d}-{row['container_id']}-real.wav", row)
        )
    for row in previous["synthetic"]:
        pending.append((row["kind"], f"synthetic-{row['kind']}-input.wav", row))
    output.mkdir(parents=True)
    report = {
        "status": "running",
        "model": provenance,
        "reference_audio_input": True,
        "scope": "independent weights/architecture, overlapping source training data; not independent test or naturalness authority",
        "rows": [],
        "wavs": [],
    }
    preview = []
    for idx, (name, filename, prior) in enumerate(pending):
        record = records[filename]
        path = Path(record["wav"])
        if hashlib.sha256(path.read_bytes()).hexdigest() != record["sha256"]:
            raise ValueError("input waveform identity mismatch")
        wave, rate = sf.read(path, dtype="float32")
        if rate != p.RATE:
            raise ValueError("wrong input rate")
        for variant, audio in [
            ("original", wave),
            ("reversed", wave[::-1].copy()),
            ("shuffled", shuffle_blocks(wave)),
        ]:
            result, probabilities = infer(model, extractor, audio)
            result.update(
                {
                    "id": name,
                    "variant": variant,
                    "input_wav": str(path),
                    "input_sha256": record["sha256"],
                }
            )
            np.savez_compressed(
                output / f"{name}-{variant}-posterior.npz", probabilities=probabilities
            )
            if "kind" in prior and name != "noise" and variant == "original":
                _, times, targets = ridge.synthetic(name)
                truth = np.stack(
                    [np.interp(result["time_seconds"], times, hz) for hz in targets]
                )
                error = np.min(
                    abs(1200 * np.log2(np.array(result["axial_hz"])[None] / truth)),
                    axis=0,
                )[4:-4]
                result.update(
                    {
                        "median_error_cents": float(np.median(error)),
                        "within50cents_fraction": float(np.mean(error < 50)),
                    }
                )
            report["rows"].append(result)
            if variant == "original":
                extracted = component(audio, result["time_seconds"], result["axial_hz"])
                result["component_energy_fraction"] = float(
                    np.mean(extracted**2) / max(np.mean(audio**2), 1e-20)
                )
                path = output / f"{name}-neural-component.wav"
                sf.write(path, extracted, p.RATE, subtype="PCM_16")
                report["wavs"].append(
                    {
                        "wav": str(path),
                        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                        "gain": 1,
                    }
                )
                if idx < 2:
                    classic_path = probe / filename.replace("-real.wav", "-ridge.wav")
                    classic, _ = sf.read(classic_path, dtype="float32")
                    for sound in [audio, classic, extracted]:
                        preview.extend([sound, np.zeros(p.RATE // 2)])
        print({"completed": idx + 1, "total": len(pending)}, flush=True)
    path = output / "comparison.wav"
    sf.write(path, np.concatenate(preview), p.RATE, subtype="PCM_16")
    report["wavs"].append(
        {
            "wav": str(path),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "gain": 1,
        }
    )
    report["comparison_order"] = (
        "first two training objects: real/classical-ridge/neural-pitch-band, gain1; source-dependent diagnostics"
    )
    report["status"] = "complete"
    p.save_json(output / "result.json", report)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("model-dir", "probe", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    run(args.model_dir, args.probe, args.output)
