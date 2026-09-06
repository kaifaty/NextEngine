"""Disclosed glass audio -> modal fit -> small encoder -> report-only WAVs.

This is an in-sample reconstruction experiment, not a physical parameter
estimator, admission validator, or replacement for any frozen study.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import subprocess
import time
from collections.abc import Callable
from itertools import pairwise
from pathlib import Path

import numpy as np
import torch
from scipy import signal
from scipy.io import wavfile
from torch import nn
from torch.nn import functional as F

RATE = 32_000
SAMPLES = 48_000
MODES = 32
BANDS = 8
SEED = 42
PARAMETERS = MODES * 5 + BANDS
SOURCES = (
    (
        "761160",
        0.58,
        "f477d0e2de75bdf81e3e2ec6b2e220b7faa0466412073d547237b13da7711085",
    ),
    (
        "761161",
        0.77,
        "4d371e9d9583796b524e664093aeedb295006da8a05f143cb7409ce8e7ee5991",
    ),
    (
        "761162",
        0.95,
        "7be43b4a78f3a9775e62225cd3c96d1eeab1c3f975c8e4a6650d0e1c2a029065",
    ),
)


def validate_audio(value: np.ndarray) -> np.ndarray:
    value = np.asarray(value, dtype=np.float32)
    if value.ndim != 1 or not len(value) or not np.isfinite(value).all():
        raise ValueError("audio must be a finite, nonempty mono signal")
    if np.max(np.abs(value)) < 1e-8:
        raise ValueError("audio is silent")
    return value


def decode(path: Path) -> np.ndarray:
    result = subprocess.run(
        [
            "ffmpeg",
            "-v",
            "error",
            "-i",
            str(path),
            "-t",
            "30",
            "-f",
            "f32le",
            "-ac",
            "1",
            "-ar",
            str(RATE),
            "pipe:1",
        ],
        capture_output=True,
        timeout=30,
        check=False,
    )
    if result.returncode:
        raise ValueError(
            f"cannot decode {path.name}: {result.stderr.decode(errors='replace')[:300]}"
        )
    return validate_audio(np.frombuffer(result.stdout, dtype="<f4").copy())


def load_sources(root: Path) -> tuple[torch.Tensor, list[int], list[dict]]:
    targets, onsets, records = [], [], []
    for sound_id, peak_seconds, expected in SOURCES:
        path = root / f"{sound_id}_13431397-hq.mp3"
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            raise ValueError(f"source hash mismatch: {sound_id}")
        decoded = decode(path)
        start = round((peak_seconds - 0.05) * RATE)
        crop = decoded[start : start + SAMPLES].copy()
        if len(crop) != SAMPLES:
            raise ValueError(f"short source: {sound_id}")
        crop -= crop.mean()
        scale = 0.8 / float(np.max(np.abs(crop)))
        crop *= scale
        # Detect attack inside the known first-hit crop; alignment is metadata,
        # not a sampled transient passed to the synthesizer.
        rms = np.sqrt(np.convolve(crop[:3200] ** 2, np.ones(32) / 32, mode="same"))
        onset = int(np.flatnonzero(rms > rms.max() * 0.1)[0])
        targets.append(validate_audio(crop))
        onsets.append(onset)
        records.append(
            {
                "sound_id": sound_id,
                "path": str(path.resolve()),
                "sha256": actual,
                "url": f"https://freesound.org/people/wasserbjorn/sounds/{sound_id}/",
                "license": "CC0-1.0",
                "role": "generator_train",
                "crop_start_seconds": start / RATE,
                "crop_samples": SAMPLES,
                "source_decoded_peak": float(np.max(np.abs(decoded))),
                "normalization_gain": scale,
                "onset_sample": onset,
            }
        )
    return torch.tensor(np.stack(targets)), onsets, records


def logit(x: np.ndarray) -> np.ndarray:
    x = np.clip(x, 1e-6, 1 - 1e-6)
    return np.log(x / (1 - x))


def physical(raw: torch.Tensor) -> tuple[torch.Tensor, ...]:
    modal = raw[..., : MODES * 5].reshape(*raw.shape[:-1], MODES, 5)
    frequency = 80 * torch.exp(torch.sigmoid(modal[..., 0]) * math.log(15000 / 80))
    t60 = 0.02 * torch.exp(torch.sigmoid(modal[..., 1]) * math.log(4 / 0.02))
    amplitude = F.softplus(modal[..., 2])
    phase = F.normalize(modal[..., 3:5], dim=-1, eps=1e-8)
    noise = F.softplus(raw[..., MODES * 5 :])
    return frequency, t60, amplitude, phase, noise


class Synthesizer(nn.Module):
    def __init__(self, samples: int = SAMPLES):
        super().__init__()
        self.register_buffer("clock", torch.arange(samples, dtype=torch.float32) / RATE)
        rng = np.random.default_rng(SEED)
        noise = rng.standard_normal(round(0.02 * RATE))
        spectrum = np.fft.rfft(noise)
        frequencies = np.fft.rfftfreq(len(noise), 1 / RATE)
        edges = np.geomspace(80, 15000, BANDS + 1)
        banks = []
        for lo, hi in pairwise(edges):
            band = np.fft.irfft(
                spectrum * ((frequencies >= lo) & (frequencies < hi)), n=len(noise)
            )
            band /= max(float(np.sqrt(np.mean(band**2))), 1e-8)
            envelope = np.sin(np.linspace(0, np.pi, len(noise))) ** 2
            banks.append(band * envelope)
        self.register_buffer(
            "attack", torch.tensor(np.array(banks), dtype=torch.float32)
        )

    def forward(self, raw: torch.Tensor, onsets: list[int]) -> torch.Tensor:
        if raw.ndim == 1:
            raw = raw[None]
        if raw.shape != (len(onsets), PARAMETERS):
            raise ValueError("invalid parameter or onset shape")
        frequency, t60, amplitude, phase, noise = physical(raw)
        t = (
            self.clock[None, :]
            - torch.tensor(onsets, dtype=torch.float32)[:, None] / RATE
        )
        active = t >= 0
        t = t.clamp_min(0)
        angle = 2 * math.pi * frequency[..., None] * t[:, None, :]
        oscillator = (
            torch.sin(angle) * phase[..., 1, None]
            + torch.cos(angle) * phase[..., 0, None]
        )
        modal = (
            amplitude[..., None]
            * torch.exp(-math.log(1000) * t[:, None, :] / t60[..., None])
            * oscillator
        ).sum(1)
        modal = modal * active
        attacks = noise @ self.attack
        padded = []
        for row, onset in zip(attacks, onsets):
            if onset < 0 or onset + len(row) > len(self.clock):
                raise ValueError("attack falls outside render window")
            padded.append(F.pad(row, (onset, len(self.clock) - onset - len(row))))
        return modal + torch.stack(padded)


def initialize(target: np.ndarray, onset: int) -> torch.Tensor:
    tail = target[onset:]
    n_fft = 65536
    spectrum = np.abs(
        np.fft.rfft(tail * np.exp(-np.arange(len(tail)) / RATE / 0.25), n=n_fft)
    )
    hz = np.fft.rfftfreq(n_fft, 1 / RATE)
    peaks, _ = signal.find_peaks(spectrum, distance=round(20 / (RATE / n_fft)))
    peaks = peaks[(hz[peaks] >= 80) & (hz[peaks] <= 15000)]
    selected = peaks[np.argsort(spectrum[peaks])[-MODES:]]
    if len(selected) < MODES:
        raise ValueError("fewer than 32 separated spectral peaks")
    frequency = np.sort(hz[selected])
    # Estimate each decay from the log-magnitude STFT ridge, then solve the
    # linear sine/cosine gains. This is an initializer, not extra source audio.
    bins, times, stft = signal.stft(
        tail, RATE, nperseg=2048, noverlap=1536, boundary=None
    )
    t60 = []
    for f in frequency:
        magnitude = np.abs(stft[np.argmin(abs(bins - f))])
        mask = (times >= 0.04) & (magnitude > magnitude.max() * 0.05)
        slope = (
            np.polyfit(times[mask], np.log(magnitude[mask] + 1e-9), 1)[0]
            if mask.sum() >= 3
            else -20.0
        )
        t60.append(np.clip(-math.log(1000) / min(slope, -1e-3), 0.021, 3.99))
    t60 = np.array(t60)
    t = np.maximum(np.arange(len(target)) / RATE - onset / RATE, 0)
    active = (np.arange(len(target)) >= onset)[:, None]
    envelope = np.exp(-math.log(1000) * t[:, None] / t60) * active
    angles = 2 * np.pi * t[:, None] * frequency
    basis = np.concatenate(
        (envelope * np.sin(angles), envelope * np.cos(angles)), axis=1
    )
    coefficients = np.linalg.lstsq(basis, target, rcond=1e-5)[0]
    sine, cosine = coefficients[:MODES], coefficients[MODES:]
    amplitude = np.maximum(np.hypot(sine, cosine), 1e-6)
    modal = np.stack(
        (
            logit(np.log(frequency / 80) / math.log(15000 / 80)),
            logit(np.log(t60 / 0.02) / math.log(4 / 0.02)),
            np.log(np.expm1(amplitude)),
            cosine / amplitude,
            sine / amplitude,
        ),
        axis=1,
    )
    return torch.tensor(
        np.concatenate((modal.ravel(), np.full(BANDS, -7.0))), dtype=torch.float32
    )


class AudioLoss(nn.Module):
    def __init__(self, target: torch.Tensor):
        super().__init__()
        self.sizes = (512, 2048, 8192)
        self.target = target
        self.windows = [torch.hann_window(size) for size in self.sizes]
        self.spectra = [
            self.spectrum(target, size, window)
            for size, window in zip(self.sizes, self.windows)
        ]
        self.envelope = self.rms(target)

    @staticmethod
    def spectrum(x: torch.Tensor, size: int, window: torch.Tensor) -> torch.Tensor:
        return torch.stft(
            x, size, hop_length=size // 4, window=window, return_complex=True
        ).abs()

    @staticmethod
    def rms(x: torch.Tensor) -> torch.Tensor:
        return torch.sqrt(F.avg_pool1d(x[:, None].square(), 320, 320)[:, 0] + 1e-10)

    def parts(self, prediction: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        spectral = []
        for size, window, target in zip(self.sizes, self.windows, self.spectra):
            value = self.spectrum(prediction, size, window)
            convergence = (value - target).flatten(1).norm(dim=1) / target.flatten(
                1
            ).norm(dim=1).clamp_min(1e-8)
            log_distance = (
                (torch.log1p(10 * value) - torch.log1p(10 * target)).abs().mean((1, 2))
            )
            spectral.append(convergence + log_distance)
        envelope = (self.rms(prediction) - self.envelope).abs().mean(
            1
        ) / self.envelope.mean(1).clamp_min(1e-8)
        return torch.stack(spectral).mean(0), envelope

    def forward(self, prediction: torch.Tensor) -> torch.Tensor:
        spectral, envelope = self.parts(prediction)
        return (spectral + 0.1 * envelope).mean()


def features(target: torch.Tensor) -> torch.Tensor:
    spectrum = AudioLoss.spectrum(target, 2048, torch.hann_window(2048))
    return F.interpolate(
        torch.log1p(10 * spectrum)[:, None],
        size=(64, 32),
        mode="bilinear",
        align_corners=False,
    ).flatten(1)


class Encoder(nn.Module):
    def __init__(self, inputs: torch.Tensor, parameters: torch.Tensor):
        super().__init__()
        self.register_buffer("input_mean", inputs.mean(0))
        self.register_buffer(
            "input_scale", inputs.std(0, unbiased=False).clamp_min(0.05)
        )
        self.register_buffer("output_mean", parameters.mean(0))
        self.register_buffer(
            "output_scale", parameters.std(0, unbiased=False).clamp_min(0.05)
        )
        self.net = nn.Sequential(
            nn.Linear(2048, 128),
            nn.SiLU(),
            nn.Linear(128, 128),
            nn.SiLU(),
            nn.Linear(128, PARAMETERS),
        )

    def forward(self, inputs: torch.Tensor) -> torch.Tensor:
        return self.output_mean + self.output_scale * self.net(
            (inputs - self.input_mean) / self.input_scale
        )


def optimize(
    module: nn.Module,
    objective: Callable[[], torch.Tensor],
    optimizer: torch.optim.Optimizer,
    steps: int,
    deadline: float,
    every: int,
    publish: Callable[[int, float], None],
) -> dict:
    with torch.no_grad():
        initial = float(objective())
    if not math.isfinite(initial):
        raise ValueError("nonfinite initial loss")
    best, best_state, best_step = initial, copy.deepcopy(module.state_dict()), 0
    publish(0, initial)
    status, completed = "complete", 0
    try:
        for step in range(1, steps + 1):
            if time.monotonic() >= deadline:
                status = "time_limit"
                break
            optimizer.zero_grad(set_to_none=True)
            loss = objective()
            if not torch.isfinite(loss):
                status = "nonfinite_loss"
                break
            loss.backward()
            if any(
                p.grad is not None and not torch.isfinite(p.grad).all()
                for p in module.parameters()
            ):
                status = "nonfinite_gradient"
                break
            torch.nn.utils.clip_grad_norm_(module.parameters(), 10.0)
            optimizer.step()
            completed = step
            with torch.no_grad():
                value = float(objective())
            if not math.isfinite(value):
                status = "nonfinite_update"
                break
            if value < best:
                best, best_state, best_step = (
                    value,
                    copy.deepcopy(module.state_dict()),
                    step,
                )
            if step % every == 0:
                current = copy.deepcopy(module.state_dict())
                module.load_state_dict(best_state)
                publish(step, best)
                module.load_state_dict(current)
    except KeyboardInterrupt:
        status = "interrupted"
    module.load_state_dict(best_state)
    publish(completed, best)
    return {
        "initial_loss": initial,
        "best_loss": best,
        "best_step": best_step,
        "steps": completed,
        "status": status,
    }


def write_wav(path: Path, value: np.ndarray) -> None:
    value = validate_audio(value)
    if np.max(np.abs(value)) > 0.99001:
        raise ValueError("use a common comparison gain before writing WAV")
    temporary = path.with_suffix(".partial.wav")
    wavfile.write(temporary, RATE, np.rint(value * 32767).astype(np.int16))
    temporary.replace(path)


def write_json(path: Path, value: dict) -> None:
    temporary = path.with_suffix(".partial.json")
    temporary.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")
    temporary.replace(path)


def diagnostics(value: np.ndarray) -> dict:
    magnitude = np.abs(np.fft.rfft(value))
    centroid = float(
        np.dot(np.fft.rfftfreq(len(value), 1 / RATE), magnitude)
        / max(magnitude.sum(), 1e-12)
    )
    energy = np.cumsum(value[::-1].astype(np.float64) ** 2)[::-1]
    db = 10 * np.log10(np.maximum(energy / max(energy[0], 1e-12), 1e-12))
    selected = (db <= -5) & (db >= -25)
    slope = (
        np.polyfit(np.arange(len(value))[selected] / RATE, db[selected], 1)[0]
        if selected.sum() >= 3
        else 0
    )
    return {
        "peak": float(np.max(np.abs(value))),
        "rms": float(np.sqrt(np.mean(value**2))),
        "spectral_centroid_hz": centroid,
        "t20_seconds": float(-20 / slope) if slope < 0 else None,
    }


def publish_audio(
    output: Path,
    targets: torch.Tensor,
    direct: torch.Tensor,
    neural: torch.Tensor | None,
) -> dict:
    variants = {"original": targets, "direct": direct}
    if neural is not None:
        variants["neural"] = neural
    paths, gains, combined = {}, [], []
    for index, (sound_id, _, _) in enumerate(SOURCES):
        peak = max(float(value[index].abs().max()) for value in variants.values())
        gain = min(1.0, 0.98 / max(peak, 1e-8))
        gains.append(gain)
        for label, value in variants.items():
            path = output / f"{sound_id}-{label}.wav"
            audio = value[index].detach().numpy() * gain
            write_wav(path, audio)
            paths[f"{sound_id}-{label}"] = str(path)
            combined.extend((audio, np.zeros(RATE // 2, dtype=np.float32)))
    comparison = output / "comparison.wav"
    write_wav(comparison, np.concatenate(combined))
    paths["comparison"] = str(comparison)
    return {
        "paths": paths,
        "common_gain_per_example": gains,
        "comparison_order": list(variants),
        "pause_seconds": 0.5,
    }


def run(source_root: Path, output: Path, budget_seconds: float) -> dict:
    if not math.isfinite(budget_seconds) or budget_seconds <= 0:
        raise ValueError("budget must be positive and finite")
    output = output.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output.is_relative_to(repository):
        raise ValueError("generated artifacts must stay outside the repository")
    if output.exists():
        raise ValueError("output must be a new directory")
    torch.set_num_threads(4)
    torch.manual_seed(SEED)
    start = time.monotonic()
    targets, onsets, records = load_sources(source_root)
    synth = Synthesizer()
    initial = torch.stack(
        [initialize(row.numpy(), onset) for row, onset in zip(targets, onsets)]
    )
    output.mkdir(parents=True)
    parameters = initial.clone()
    report = {
        "claim": "REPORT_ONLY / THREE_TRAIN_EXAMPLES / NO_GENERALIZATION_OR_ADMISSION",
        "seed": SEED,
        "budget_seconds": budget_seconds,
        "device": "cpu",
        "torch": torch.__version__,
        "sample_rate": RATE,
        "samples": SAMPLES,
        "modes": MODES,
        "noise_bands": BANDS,
        "sources": records,
        "source_samples_mixed_into_output": False,
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "direct": [],
        "status": "direct_fit_running",
    }

    def save(neural: torch.Tensor | None = None) -> None:
        with torch.no_grad():
            report["audio"] = publish_audio(
                output, targets, synth(parameters, onsets), neural
            )
        report["elapsed_seconds"] = time.monotonic() - start
        write_json(output / "result.json", report)

    save()
    # Keep the initial fit available as a separate diagnostic, using its own
    # paired target gain rather than independently peak-normalizing it.
    with torch.no_grad():
        initial_audio = synth(initial, onsets)
        initial_directory = output / "initial-direct"
        initial_directory.mkdir()
        report["initial_direct_audio"] = publish_audio(
            initial_directory, targets, initial_audio, None
        )
    initial_loss = float(AudioLoss(targets)(initial_audio))
    report["direct_initial_acoustic_loss"] = initial_loss
    direct_budget = min(600.0, budget_seconds / 2)
    for row in range(len(targets)):
        # Separate groups keep sub-Hz frequency updates small while allowing
        # envelope and gain parameters to move at a useful rate.
        vector = initial[row].clone()
        frequency_indices = torch.arange(0, MODES * 5, 5)
        other_indices = torch.tensor(
            [i for i in range(PARAMETERS) if i % 5 != 0 or i >= MODES * 5]
        )
        model = nn.ParameterDict(
            {
                "frequency": nn.Parameter(vector[frequency_indices].clone()),
                "other": nn.Parameter(vector[other_indices].clone()),
            }
        )

        def current(
            vector=vector,
            frequency_indices=frequency_indices,
            model=model,
            other_indices=other_indices,
        ) -> torch.Tensor:
            return vector.scatter(0, frequency_indices, model["frequency"]).scatter(
                0, other_indices, model["other"]
            )

        loss = AudioLoss(targets[row : row + 1])

        def publish(step: int, value: float, row=row, current=current) -> None:
            parameters[row] = current().detach()
            report["progress"] = {
                "stage": "direct",
                "sound_id": SOURCES[row][0],
                "step": step,
                "loss": value,
            }
            save()
            print(json.dumps(report["progress"]), flush=True)

        optimizer = torch.optim.Adam(
            [
                {"params": [model["frequency"]], "lr": 1e-5},
                {"params": [model["other"]], "lr": 0.02},
            ]
        )
        info = optimize(
            model,
            lambda loss=loss, current=current, row=row: loss(
                synth(current(), [onsets[row]])
            ),
            optimizer,
            500,
            start + direct_budget * (row + 1) / 3,
            50,
            publish,
        )
        report["direct"].append(info)
        if info["status"] == "interrupted":
            report["status"] = "interrupted_after_direct"
            save()
            return report

    inputs = features(targets)
    encoder = Encoder(inputs, parameters)
    acoustic = AudioLoss(targets)
    with torch.no_grad():
        neural_initial = synth(encoder(inputs), onsets)
        report["neural_initial_acoustic_loss"] = float(acoustic(neural_initial))
        initial_directory = output / "initial-neural"
        initial_directory.mkdir()
        report["initial_neural_audio"] = publish_audio(
            initial_directory, targets, synth(parameters, onsets), neural_initial
        )
    report["status"] = "encoder_running"

    def publish_neural(step: int, value: float) -> None:
        with torch.no_grad():
            predicted = synth(encoder(inputs), onsets)
            report["progress"] = {
                "stage": report["status"],
                "step": step,
                "loss": value,
                "acoustic_loss": float(acoustic(predicted)),
            }
        temporary = output / "encoder.partial.pt"
        torch.save(encoder.state_dict(), temporary)
        temporary.replace(output / "encoder.pt")
        save(predicted)
        print(json.dumps(report["progress"]), flush=True)

    objective = lambda: (
        ((encoder(inputs) - parameters) / encoder.output_scale).square().mean()
    )
    report["encoder"] = optimize(
        encoder,
        objective,
        torch.optim.Adam(encoder.parameters(), lr=0.001),
        2000,
        start + budget_seconds * 0.9,
        100,
        publish_neural,
    )
    if report["encoder"]["status"] != "interrupted":
        report["status"] = "spectral_finetune_running"
        report["finetune"] = optimize(
            encoder,
            lambda: acoustic(synth(encoder(inputs), onsets)),
            torch.optim.Adam(encoder.parameters(), lr=1e-5),
            100,
            start + budget_seconds,
            50,
            publish_neural,
        )
    with torch.no_grad():
        predicted_parameters = encoder(inputs)
        predicted = synth(predicted_parameters, onsets)
        direct = synth(parameters, onsets)
        direct_spectral, direct_envelope = acoustic.parts(direct)
        neural_spectral, neural_envelope = acoustic.parts(predicted)
        final_loss = float(acoustic(predicted))
    # Reload the persisted weights in a separate model instance before declaring
    # a usable neural artifact. The checkpoint contains tensors only.
    reloaded = Encoder(inputs, parameters)
    reloaded.load_state_dict(
        torch.load(output / "encoder.pt", map_location="cpu", weights_only=True)
    )
    with torch.no_grad():
        reload_equal = bool(torch.equal(synth(reloaded(inputs), onsets), predicted))
    stages = [*report["direct"], report["encoder"]]
    if "finetune" in report:
        stages.append(report["finetune"])
    report.update(
        {
            "status": "complete"
            if all(stage["status"] == "complete" for stage in stages)
            else "partial",
            "reload_exact": reload_equal,
            "neural_final_acoustic_loss": final_loss,
            "technical_success": bool(
                reload_equal
                and final_loss < report["neural_initial_acoustic_loss"]
                and torch.all(neural_spectral <= 1.1 * direct_spectral)
            ),
            "metrics": [
                {
                    "sound_id": record["sound_id"],
                    "direct_spectral": float(direct_spectral[i]),
                    "neural_spectral": float(neural_spectral[i]),
                    "direct_envelope": float(direct_envelope[i]),
                    "neural_envelope": float(neural_envelope[i]),
                    "neural_to_direct_spectral_ratio": float(
                        neural_spectral[i] / direct_spectral[i].clamp_min(1e-8)
                    ),
                    "original": diagnostics(targets[i].numpy()),
                    "direct": diagnostics(direct[i].numpy()),
                    "neural": diagnostics(predicted[i].numpy()),
                }
                for i, record in enumerate(records)
            ],
        }
    )
    write_json(
        output / "parameters.json",
        {
            "representation": "32_damped_modes_and_8_noise_bands",
            "onsets": onsets,
            "direct_raw": parameters.tolist(),
            "neural_raw": predicted_parameters.tolist(),
        },
    )
    save(predicted)
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--budget-seconds", type=float, default=1200)
    args = parser.parse_args()
    report = run(args.source_root, args.output, args.budget_seconds)
    print(
        json.dumps(
            {
                "status": report["status"],
                "result": str(args.output / "result.json"),
                "technical_success": report.get("technical_success", False),
            }
        ),
        flush=True,
    )


if __name__ == "__main__":
    main()
